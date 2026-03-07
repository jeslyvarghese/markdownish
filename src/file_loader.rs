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

const MAX_FILE_SIZE: u64 = 50 * 1024 * 1024; // 50MB

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
            Some(ext) => matches!(ext.to_lowercase().as_str(), "md" | "markdown" | "mdown" | "mkd" | "mdx"),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_valid_markdown_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "# Hello\n\nWorld").unwrap();
        let content = FileLoader::load_file(file.path()).unwrap();
        assert!(content.contains("# Hello"));
    }

    #[test]
    fn test_load_nonexistent_file_returns_error() {
        let result = FileLoader::load_file(Path::new("/tmp/this_does_not_exist_12345.md"));
        assert!(result.is_err());
    }

    #[test]
    fn test_is_markdown_extension_md() {
        assert!(FileLoader::is_markdown_file(Path::new("test.md")));
    }

    #[test]
    fn test_is_markdown_extension_markdown() {
        assert!(FileLoader::is_markdown_file(Path::new("test.markdown")));
    }

    #[test]
    fn test_is_markdown_extension_case_insensitive() {
        assert!(FileLoader::is_markdown_file(Path::new("test.MD")));
        assert!(FileLoader::is_markdown_file(Path::new("test.Md")));
    }

    #[test]
    fn test_not_markdown_extension() {
        assert!(!FileLoader::is_markdown_file(Path::new("test.txt")));
        assert!(!FileLoader::is_markdown_file(Path::new("test.rs")));
        assert!(!FileLoader::is_markdown_file(Path::new("test.html")));
    }

    #[test]
    fn test_no_extension() {
        assert!(!FileLoader::is_markdown_file(Path::new("README")));
    }

    #[test]
    fn test_file_title_extracts_filename() {
        assert_eq!(FileLoader::file_title(Path::new("/home/user/notes.md")), "notes.md");
    }

    #[test]
    fn test_file_title_root_path() {
        assert_eq!(FileLoader::file_title(Path::new("/")), "Untitled");
    }

    #[test]
    fn test_load_utf8_content() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "# Unicode: 日本語 emoji 🦀").unwrap();
        let content = FileLoader::load_file(file.path()).unwrap();
        assert!(content.contains("日本語"));
        assert!(content.contains("🦀"));
    }
}
