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
    let home = dirs_next()
        .expect("could not determine home directory");
    home.join("Library")
        .join("Application Support")
        .join("com.murmur.voice")
        .join("models")
}

pub(crate) fn model_path() -> PathBuf {
    model_dir().join("ggml-large-v3-turbo.bin")
}

pub(crate) fn is_model_ready() -> bool {
    let path = model_path();
    match std::fs::metadata(&path) {
        Ok(meta) => meta.len() == EXPECTED_SIZE,
        Err(_) => false,
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

fn dirs_next() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}
