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
}

impl Error {
    /// Exit codes per SPECS.md Section 10.1:
    /// - PathNotFound/NotADirectory/NoSourceFiles: 1
    /// - ParseError/GitError: 2
    /// - ConfigError: 3
    /// - IoError: 4
    pub fn exit_code(&self) -> i32 {
        match self {
            Error::FileNotFound(_) => 1,
            Error::NotGitRepo | Error::UnsupportedLanguage(_) | Error::AnalysisError(_) => 2,
            Error::Git(_) => 2,
            Error::ConfigError(_) | Error::Toml(_) => 3,
            Error::Io(_) | Error::Json(_) => 4,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit_code_file_not_found() {
        // Per SPECS.md Section 10.1: PathNotFound = 1
        let err = Error::FileNotFound(PathBuf::from("/test"));
        assert_eq!(err.exit_code(), 1);
    }

    #[test]
    fn test_exit_code_not_git_repo() {
        // Per SPECS.md Section 10.1: GitError = 2
        let err = Error::NotGitRepo;
        assert_eq!(err.exit_code(), 2);
    }

    #[test]
    fn test_exit_code_unsupported_language() {
        // Treat as parse/analysis error = 2
        let err = Error::UnsupportedLanguage("cobol".to_string());
        assert_eq!(err.exit_code(), 2);
    }

    #[test]
    fn test_exit_code_config() {
        // Per SPECS.md Section 10.1: ConfigError = 3
        let err = Error::ConfigError("bad config".to_string());
        assert_eq!(err.exit_code(), 3);
    }

    #[test]
    fn test_exit_code_analysis() {
        // Analysis/parse errors = 2
        let err = Error::AnalysisError("failed".to_string());
        assert_eq!(err.exit_code(), 2);
    }

    #[test]
    fn test_exit_code_io() {
        // Per SPECS.md Section 10.1: IoError = 4
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file");
        let err: Error = io_err.into();
        assert_eq!(err.exit_code(), 4);
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
