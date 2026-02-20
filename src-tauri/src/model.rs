use std::path::{Path, PathBuf};
#[cfg(target_os = "windows")]
use std::sync::Once;
use thiserror::Error;
use futures_util::StreamExt;
use tokio::io::AsyncWriteExt;

const MODEL_URL: &str =
    "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin";
const EXPECTED_SIZE: u64 = 1_624_555_275;

#[derive(Debug, Error)]
pub(crate) enum ModelError {
    #[error("failed to create model directory: {0}")]
    CreateDir(String),
    #[error("download failed: {0}")]
    Download(String),
    #[error("file size mismatch: expected {expected}, got {actual}")]
    SizeMismatch { expected: u64, actual: u64 },
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

impl serde::Serialize for ModelError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub(crate) fn model_dir(base: &Path) -> PathBuf {
    base.join("models")
}

pub(crate) fn model_path(base: &Path) -> PathBuf {
    model_dir(base).join("ggml-large-v3-turbo.bin")
}

pub(crate) fn is_model_ready(base: &Path) -> bool {
    // On Windows, migrate model from old macOS-style path (runs at most once per process)
    #[cfg(target_os = "windows")]
    {
        static MIGRATE: Once = Once::new();
        let base_owned = base.to_path_buf();
        MIGRATE.call_once(move || migrate_model_from_old_path(&base_owned));
    }

    let path = model_path(base);
    match std::fs::metadata(&path) {
        Ok(meta) => meta.len() == EXPECTED_SIZE,
        Err(_) => false,
    }
}

/// On Windows, earlier versions stored the model under $HOME/Library/Application Support/...
/// (a macOS path). Migrate it to the correct $APPDATA/... location if found.
#[cfg(target_os = "windows")]
fn migrate_model_from_old_path(base: &Path) {
    let new_path = model_path(base);
    if new_path.exists() {
        return; // already at correct location
    }

    let old_path = std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|h| {
            h.join("Library")
                .join("Application Support")
                .join("com.murmur.voice")
                .join("models")
                .join("ggml-large-v3-turbo.bin")
        });

    if let Some(old) = old_path {
        if old.exists() {
            if let Some(parent) = new_path.parent() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    log::error!("failed to create model directory {:?}: {}", parent, e);
                    return;
                }
            }
            if let Err(e) = std::fs::rename(&old, &new_path) {
                // Cross-drive move: rename fails, fall back to copy + delete
                log::warn!("rename failed (likely cross-drive): {}, trying copy", e);
                if std::fs::copy(&old, &new_path).is_ok() {
                    let _ = std::fs::remove_file(&old);
                    log::info!("copied model from {:?} to {:?}", old, new_path);
                } else {
                    log::error!("failed to copy model from {:?} to {:?}", old, new_path);
                }
            } else {
                log::info!("migrated model from {:?} to {:?}", old, new_path);
            }
        }
    }
}

pub(crate) async fn download_model<F>(base: &Path, progress_callback: F) -> Result<(), ModelError>
where
    F: Fn(u64, u64),
{
    let dir = model_dir(base);
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| ModelError::CreateDir(e.to_string()))?;

    let path = model_path(base);

    let client = reqwest::Client::new();
    let response = client
        .get(MODEL_URL)
        .send()
        .await
        .map_err(|e| ModelError::Download(e.to_string()))?;

    let total_size = response.content_length().unwrap_or(EXPECTED_SIZE);

    let mut file = tokio::fs::File::create(&path).await?;
    let stream = response.bytes_stream();

    copy_stream_with_progress(stream, &mut file, total_size, progress_callback).await?;

    file.sync_all().await?;

    // Verify size
    let actual_size = tokio::fs::metadata(&path).await?.len();
    if actual_size != EXPECTED_SIZE {
        return Err(ModelError::SizeMismatch {
            expected: EXPECTED_SIZE,
            actual: actual_size,
        });
    }

    Ok(())
}

async fn copy_stream_with_progress<S, W, F, E, B>(
    mut stream: S,
    mut writer: W,
    total_size: u64,
    progress_callback: F,
) -> Result<(), ModelError>
where
    S: futures_util::Stream<Item = Result<B, E>> + Unpin,
    B: AsRef<[u8]>,
    W: tokio::io::AsyncWrite + Unpin,
    F: Fn(u64, u64),
    E: std::fmt::Display,
{
    let mut downloaded: u64 = 0;
    let mut last_reported: u64 = 0;
    // Throttle progress updates to every 1MB to avoid flooding the event loop
    const REPORT_THRESHOLD: u64 = 1_000_000;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| ModelError::Download(e.to_string()))?;
        let data = chunk.as_ref();
        writer.write_all(data).await?;
        downloaded += data.len() as u64;

        if downloaded == total_size || downloaded.saturating_sub(last_reported) >= REPORT_THRESHOLD {
            progress_callback(downloaded, total_size);
            last_reported = downloaded;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::stream;

    #[tokio::test]
    async fn test_throttling_logic() {
        let chunk_size = 100_000; // 100KB
        let num_chunks = 25; // 2.5MB total
        let total_size = (chunk_size * num_chunks) as u64;
        let data = vec![0u8; chunk_size];

        // Create a stream of 25 chunks
        let stream = stream::iter((0..num_chunks).map(|_| Ok::<_, std::io::Error>(data.clone())));

        // Use a sink writer (discards data)
        let writer = tokio::io::sink();

        let callback_count = std::sync::atomic::AtomicUsize::new(0);
        let callback = |_, _| {
            callback_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        };

        copy_stream_with_progress(stream, writer, total_size, callback)
            .await
            .expect("copy failed");

        let count = callback_count.load(std::sync::atomic::Ordering::SeqCst);

        // Expected behavior:
        // 1. At 1.0MB (chunk 10) -> Call 1
        // 2. At 2.0MB (chunk 20) -> Call 2
        // 3. At 2.5MB (chunk 25/End) -> Call 3 (Total size reached)
        assert_eq!(count, 3, "Callback should be called exactly 3 times (1MB, 2MB, End)");
    }
}
