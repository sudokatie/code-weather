pub mod schema;

pub use schema::{Config, ThresholdConfig, AnalysisConfig, DisplayConfig};

use crate::error::{Error, Result};
use std::path::{Path, PathBuf};

const CONFIG_FILENAME: &str = ".code-weather.toml";

/// Load config from explicit path, discovered path, or return defaults
pub fn load(explicit_path: Option<&Path>) -> Result<Config> {
    if let Some(path) = explicit_path {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        return Ok(config);
    }
    
    if let Ok(cwd) = std::env::current_dir() {
        if let Some(path) = find_config_file(&cwd) {
            let content = std::fs::read_to_string(&path)?;
            let config: Config = toml::from_str(&content)?;
            return Ok(config);
        }
    }
    
    Ok(Config::default())
}

/// Find config file by walking up directory tree
pub fn find_config_file(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        let config_path = current.join(CONFIG_FILENAME);
        if config_path.exists() {
            return Some(config_path);
        }
        if !current.pop() {
            return None;
        }
    }
}

/// Generate config file content
pub fn generate_config(full: bool) -> String {
    if full {
        toml::to_string_pretty(&Config::default()).unwrap_or_default()
    } else {
        r#"# Code Weather Configuration

[thresholds]
sunny_coverage = 80
cloudy_coverage = 50

[analysis]
exclude = ["node_modules", "vendor", "target", ".git"]
"#.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_find_config_in_current() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join(".code-weather.toml");
        std::fs::write(&config_path, "[thresholds]").unwrap();
        
        let found = find_config_file(dir.path());
        assert_eq!(found, Some(config_path));
    }
    
    #[test]
    fn test_find_config_in_parent() {
        let parent = TempDir::new().unwrap();
        let child = parent.path().join("subdir");
        std::fs::create_dir(&child).unwrap();
        
        let config_path = parent.path().join(".code-weather.toml");
        std::fs::write(&config_path, "[thresholds]").unwrap();
        
        let found = find_config_file(&child);
        assert_eq!(found, Some(config_path));
    }
    
    #[test]
    fn test_find_config_not_found() {
        let dir = TempDir::new().unwrap();
        let found = find_config_file(dir.path());
        assert!(found.is_none());
    }
    
    #[test]
    fn test_generate_minimal_config() {
        let config = generate_config(false);
        assert!(config.contains("[thresholds]"));
        assert!(config.contains("sunny_coverage"));
    }
    
    #[test]
    fn test_generate_full_config() {
        let config = generate_config(true);
        assert!(config.contains("thresholds"));
    }
    
    #[test]
    fn test_load_default() {
        let config = load(None).unwrap();
        assert_eq!(config.thresholds.sunny_coverage, 80);
    }
}
