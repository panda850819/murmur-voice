use std::path::PathBuf;
use thiserror::Error;

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

pub(crate) fn model_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/tmp"));
        home.join("Library")
            .join("Application Support")
            .join("com.murmur.voice")
            .join("models")
    }
    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("C:\\Users\\Default\\AppData\\Roaming"));
        appdata.join("murmur-voice").join("models")
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/tmp"));
        home.join(".local")
            .join("share")
            .join("murmur-voice")
            .join("models")
    }
}

pub(crate) fn model_path() -> PathBuf {
    model_dir().join("ggml-large-v3-turbo.bin")
}

pub(crate) fn is_model_ready() -> bool {
    // On Windows, migrate model from old macOS-style path if it exists at the new location
    #[cfg(target_os = "windows")]
    migrate_model_from_old_path();

    let path = model_path();
    match std::fs::metadata(&path) {
        Ok(meta) => meta.len() == EXPECTED_SIZE,
        Err(_) => false,
    }
}

/// On Windows, earlier versions stored the model under $HOME/Library/Application Support/...
/// (a macOS path). Migrate it to the correct $APPDATA/... location if found.
#[cfg(target_os = "windows")]
fn migrate_model_from_old_path() {
    let new_path = model_path();
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

pub(crate) async fn download_model<F>(progress_callback: F) -> Result<(), ModelError>
where
    F: Fn(u64, u64),
{
    let dir = model_dir();
    std::fs::create_dir_all(&dir).map_err(|e| ModelError::CreateDir(e.to_string()))?;

    let path = model_path();

    let client = reqwest::Client::new();
    let response = client
        .get(MODEL_URL)
        .send()
        .await
        .map_err(|e| ModelError::Download(e.to_string()))?;

    let total_size = response.content_length().unwrap_or(EXPECTED_SIZE);

    let mut file = std::fs::File::create(&path)?;
    let mut downloaded: u64 = 0;

    use futures_util::StreamExt;
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| ModelError::Download(e.to_string()))?;
        use std::io::Write;
        file.write_all(&chunk)?;
        downloaded += chunk.len() as u64;
        progress_callback(downloaded, total_size);
    }

    file.sync_all()?;

    // Verify size
    let actual_size = std::fs::metadata(&path)?.len();
    if actual_size != EXPECTED_SIZE {
        return Err(ModelError::SizeMismatch {
            expected: EXPECTED_SIZE,
            actual: actual_size,
        });
    }

    Ok(())
}

