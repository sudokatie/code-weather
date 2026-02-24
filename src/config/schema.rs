use serde::{Deserialize, Serialize};

/// Main configuration struct
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub thresholds: ThresholdConfig,
    pub analysis: AnalysisConfig,
    pub display: DisplayConfig,
}

/// Thresholds for weather conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ThresholdConfig {
    /// Test coverage percentage for "sunny" (0-100)
    pub sunny_coverage: u8,
    /// Test coverage percentage for "cloudy" (0-100)
    pub cloudy_coverage: u8,
    /// Maximum cyclomatic complexity for "sunny"
    pub sunny_complexity: u8,
    /// Maximum cyclomatic complexity for "cloudy"
    pub cloudy_complexity: u8,
    /// Documentation coverage percentage for "sunny"
    pub sunny_docs: u8,
    /// Documentation coverage percentage for "cloudy"
    pub cloudy_docs: u8,
    /// Maximum function length (lines) for "sunny"
    pub sunny_fn_length: u32,
    /// Maximum function length (lines) for "cloudy"
    pub cloudy_fn_length: u32,
}

impl Default for ThresholdConfig {
    fn default() -> Self {
        Self {
            sunny_coverage: 80,
            cloudy_coverage: 50,
            sunny_complexity: 10,
            cloudy_complexity: 20,
            sunny_docs: 70,
            cloudy_docs: 30,
            sunny_fn_length: 30,
            cloudy_fn_length: 60,
        }
    }
}

/// Analysis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AnalysisConfig {
    /// Patterns to exclude from analysis
    pub exclude: Vec<String>,
    /// Patterns to include (overrides exclude)
    pub include: Vec<String>,
    /// Maximum file size to analyze (bytes)
    pub max_file_size: usize,
    /// Analyze git history
    pub analyze_git: bool,
    /// Git history depth (commits)
    pub git_depth: usize,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            exclude: vec![
                "node_modules".to_string(),
                "vendor".to_string(),
                "target".to_string(),
                ".git".to_string(),
                "dist".to_string(),
                "build".to_string(),
                "__pycache__".to_string(),
            ],
            include: vec![],
            max_file_size: 1024 * 1024, // 1MB
            analyze_git: true,
            git_depth: 100,
        }
    }
}

/// Display configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DisplayConfig {
    /// Use color output
    pub color: bool,
    /// Show ASCII art weather
    pub ascii_art: bool,
    /// Temperature unit ("celsius" or "fahrenheit")
    pub temp_unit: String,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            color: true,
            ascii_art: true,
            temp_unit: "celsius".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_thresholds() {
        let config = Config::default();
        assert_eq!(config.thresholds.sunny_coverage, 80);
        assert_eq!(config.thresholds.cloudy_coverage, 50);
        assert_eq!(config.thresholds.sunny_complexity, 10);
    }
    
    #[test]
    fn test_default_excludes() {
        let config = Config::default();
        assert!(config.analysis.exclude.contains(&"node_modules".to_string()));
        assert!(config.analysis.exclude.contains(&"target".to_string()));
    }
    
    #[test]
    fn test_parse_minimal_config() {
        let toml = r#"
[thresholds]
sunny_coverage = 90
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.thresholds.sunny_coverage, 90);
        // Other fields use defaults
        assert_eq!(config.thresholds.cloudy_coverage, 50);
    }
    
    #[test]
    fn test_parse_full_config() {
        let toml = r#"
[thresholds]
sunny_coverage = 90
cloudy_coverage = 60
sunny_complexity = 5

[analysis]
exclude = ["custom"]
analyze_git = false

[display]
color = false
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.thresholds.sunny_coverage, 90);
        assert_eq!(config.thresholds.cloudy_coverage, 60);
        assert_eq!(config.thresholds.sunny_complexity, 5);
        assert_eq!(config.analysis.exclude, vec!["custom".to_string()]);
        assert!(!config.analysis.analyze_git);
        assert!(!config.display.color);
    }
    
    #[test]
    fn test_serialize_roundtrip() {
        let config = Config::default();
        let toml = toml::to_string(&config).unwrap();
        let parsed: Config = toml::from_str(&toml).unwrap();
        assert_eq!(parsed.thresholds.sunny_coverage, config.thresholds.sunny_coverage);
    }
}
