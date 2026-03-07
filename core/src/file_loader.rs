use std::path::Path;

#[derive(Debug, Clone)]
pub struct FileLoader;

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("File is not valid UTF-8")]
    Encoding,
    #[error("File too large (max 50MB)")]
    TooLarge,
}

const MAX_FILE_SIZE: u64 = 50 * 1024 * 1024;

impl FileLoader {
    pub fn load_file(path: &Path) -> Result<String, LoadError> {
        let metadata = std::fs::metadata(path)?;
        if metadata.len() > MAX_FILE_SIZE {
            return Err(LoadError::TooLarge);
        }
        let bytes = std::fs::read(path)?;
        String::from_utf8(bytes).map_err(|_| LoadError::Encoding)
    }

    pub fn is_markdown_file(path: &Path) -> bool {
        match path.extension().and_then(|e| e.to_str()) {
            Some(ext) => matches!(
                ext.to_lowercase().as_str(),
                "md" | "markdown" | "mdown" | "mkd" | "mdx"
            ),
            None => false,
        }
    }

    pub fn file_title(path: &Path) -> String {
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled")
            .to_string()
    }
}
