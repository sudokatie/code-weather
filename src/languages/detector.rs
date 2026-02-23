use std::path::Path;

/// Supported programming languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    TypeScript,
    JavaScript,
    Python,
    Rust,
    Go,
}

impl Language {
    /// Detect language from file path
    pub fn from_path(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(Self::from_extension)
    }
    
    /// Detect language from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "ts" | "tsx" | "mts" | "cts" => Some(Language::TypeScript),
            "js" | "jsx" | "mjs" | "cjs" => Some(Language::JavaScript),
            "py" | "pyi" | "pyw" => Some(Language::Python),
            "rs" => Some(Language::Rust),
            "go" => Some(Language::Go),
            _ => None,
        }
    }
    
    /// Get all supported extensions for this language
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            Language::TypeScript => &["ts", "tsx", "mts", "cts"],
            Language::JavaScript => &["js", "jsx", "mjs", "cjs"],
            Language::Python => &["py", "pyi", "pyw"],
            Language::Rust => &["rs"],
            Language::Go => &["go"],
        }
    }
    
    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            Language::TypeScript => "TypeScript",
            Language::JavaScript => "JavaScript",
            Language::Python => "Python",
            Language::Rust => "Rust",
            Language::Go => "Go",
        }
    }
    
    /// Get tree-sitter language parser
    pub fn tree_sitter_language(&self) -> tree_sitter::Language {
        match self {
            Language::TypeScript => tree_sitter_typescript::language_typescript(),
            Language::JavaScript => tree_sitter_javascript::language(),
            Language::Python => tree_sitter_python::language(),
            Language::Rust => tree_sitter_rust::language(),
            Language::Go => tree_sitter_go::language(),
        }
    }
    
    /// Get all supported languages
    pub fn all() -> &'static [Language] {
        &[
            Language::TypeScript,
            Language::JavaScript,
            Language::Python,
            Language::Rust,
            Language::Go,
        ]
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_typescript_extensions() {
        assert_eq!(Language::from_path(Path::new("file.ts")), Some(Language::TypeScript));
        assert_eq!(Language::from_path(Path::new("file.tsx")), Some(Language::TypeScript));
        assert_eq!(Language::from_path(Path::new("file.mts")), Some(Language::TypeScript));
    }
    
    #[test]
    fn test_javascript_extensions() {
        assert_eq!(Language::from_path(Path::new("file.js")), Some(Language::JavaScript));
        assert_eq!(Language::from_path(Path::new("file.jsx")), Some(Language::JavaScript));
        assert_eq!(Language::from_path(Path::new("file.mjs")), Some(Language::JavaScript));
    }
    
    #[test]
    fn test_python_extensions() {
        assert_eq!(Language::from_path(Path::new("file.py")), Some(Language::Python));
        assert_eq!(Language::from_path(Path::new("file.pyi")), Some(Language::Python));
    }
    
    #[test]
    fn test_rust_extension() {
        assert_eq!(Language::from_path(Path::new("file.rs")), Some(Language::Rust));
    }
    
    #[test]
    fn test_go_extension() {
        assert_eq!(Language::from_path(Path::new("file.go")), Some(Language::Go));
    }
    
    #[test]
    fn test_unknown_extension() {
        assert_eq!(Language::from_path(Path::new("file.txt")), None);
        assert_eq!(Language::from_path(Path::new("file.md")), None);
    }
    
    #[test]
    fn test_no_extension() {
        assert_eq!(Language::from_path(Path::new("Makefile")), None);
    }
    
    #[test]
    fn test_case_insensitive() {
        assert_eq!(Language::from_extension("TS"), Some(Language::TypeScript));
        assert_eq!(Language::from_extension("PY"), Some(Language::Python));
        assert_eq!(Language::from_extension("RS"), Some(Language::Rust));
    }
    
    #[test]
    fn test_tree_sitter_languages() {
        // Verify they don't panic
        let _ = Language::TypeScript.tree_sitter_language();
        let _ = Language::JavaScript.tree_sitter_language();
        let _ = Language::Python.tree_sitter_language();
        let _ = Language::Rust.tree_sitter_language();
        let _ = Language::Go.tree_sitter_language();
    }
    
    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Language::TypeScript), "TypeScript");
        assert_eq!(format!("{}", Language::Python), "Python");
    }
    
    #[test]
    fn test_all_languages() {
        let all = Language::all();
        assert_eq!(all.len(), 5);
        assert!(all.contains(&Language::TypeScript));
        assert!(all.contains(&Language::Python));
    }
}
