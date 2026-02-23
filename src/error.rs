use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),
    
    #[error("Not a git repository (or any parent)")]
    NotGitRepo,
    
    #[error("Language not supported: {0}")]
    UnsupportedLanguage(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Analysis error: {0}")]
    AnalysisError(String),
    
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),
    
    #[error("Walk error: {0}")]
    Walk(#[from] walkdir::Error),
}

impl Error {
    pub fn exit_code(&self) -> i32 {
        match self {
            Error::FileNotFound(_) => 2,
            Error::NotGitRepo => 3,
            Error::UnsupportedLanguage(_) => 4,
            Error::ConfigError(_) => 5,
            _ => 1,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_exit_code_file_not_found() {
        let err = Error::FileNotFound(PathBuf::from("/test"));
        assert_eq!(err.exit_code(), 2);
    }
    
    #[test]
    fn test_exit_code_not_git_repo() {
        let err = Error::NotGitRepo;
        assert_eq!(err.exit_code(), 3);
    }
    
    #[test]
    fn test_exit_code_unsupported_language() {
        let err = Error::UnsupportedLanguage("cobol".to_string());
        assert_eq!(err.exit_code(), 4);
    }
    
    #[test]
    fn test_exit_code_config() {
        let err = Error::ConfigError("bad config".to_string());
        assert_eq!(err.exit_code(), 5);
    }
    
    #[test]
    fn test_exit_code_general() {
        let err = Error::AnalysisError("failed".to_string());
        assert_eq!(err.exit_code(), 1);
    }
    
    #[test]
    fn test_display() {
        let err = Error::UnsupportedLanguage("cobol".to_string());
        assert!(format!("{}", err).contains("cobol"));
    }
    
    #[test]
    fn test_io_from() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: Error = io_err.into();
        assert!(matches!(err, Error::Io(_)));
    }
}
