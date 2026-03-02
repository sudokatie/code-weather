pub mod schema;

pub use schema::{AnalysisConfig, Config, DisplayConfig, ThresholdConfig};

use crate::error::Result;
use std::path::{Path, PathBuf};

const CONFIG_FILENAME: &str = ".code-weather.toml";
const USER_CONFIG_DIR: &str = "code-weather";
const USER_CONFIG_FILE: &str = "config.toml";

/// Load config with full precedence (per SPECS.md Section 8.2):
/// 1. CLI flags (highest) - handled by caller
/// 2. Environment variables (CODE_WEATHER_*)
/// 3. Project config (.code-weather.toml)
/// 4. User config (~/.config/code-weather/config.toml)
/// 5. Built-in defaults (lowest)
pub fn load(explicit_path: Option<&Path>) -> Result<Config> {
    // Start with defaults
    let mut config = Config::default();

    // Layer 4: User config (~/.config/code-weather/config.toml)
    if let Some(user_config) = load_user_config() {
        merge_config(&mut config, &user_config);
    }

    // Layer 3: Project config (.code-weather.toml)
    if let Some(path) = explicit_path {
        let content = std::fs::read_to_string(path)?;
        let project_config: Config = toml::from_str(&content)?;
        merge_config(&mut config, &project_config);
    } else if let Ok(cwd) = std::env::current_dir() {
        if let Some(path) = find_config_file(&cwd) {
            let content = std::fs::read_to_string(&path)?;
            let project_config: Config = toml::from_str(&content)?;
            merge_config(&mut config, &project_config);
        }
    }

    // Layer 2: Environment variables (CODE_WEATHER_*)
    apply_env_overrides(&mut config);

    Ok(config)
}

/// Load user config from ~/.config/code-weather/config.toml
fn load_user_config() -> Option<Config> {
    let config_dir = dirs::config_dir()?;
    let user_config_path = config_dir.join(USER_CONFIG_DIR).join(USER_CONFIG_FILE);

    if user_config_path.exists() {
        let content = std::fs::read_to_string(&user_config_path).ok()?;
        toml::from_str(&content).ok()
    } else {
        None
    }
}

/// Apply environment variable overrides
/// Supports: CODE_WEATHER_SUNNY_COVERAGE, CODE_WEATHER_CLOUDY_COVERAGE,
/// CODE_WEATHER_SKIP_TESTS, CODE_WEATHER_SKIP_GIT, CODE_WEATHER_NO_COLOR, etc.
fn apply_env_overrides(config: &mut Config) {
    // Threshold overrides
    if let Ok(val) = std::env::var("CODE_WEATHER_SUNNY_COVERAGE") {
        if let Ok(v) = val.parse::<u8>() {
            config.thresholds.sunny_coverage = v;
        }
    }
    if let Ok(val) = std::env::var("CODE_WEATHER_CLOUDY_COVERAGE") {
        if let Ok(v) = val.parse::<u8>() {
            config.thresholds.cloudy_coverage = v;
        }
    }
    if let Ok(val) = std::env::var("CODE_WEATHER_SUNNY_COMPLEXITY") {
        if let Ok(v) = val.parse::<u8>() {
            config.thresholds.sunny_complexity = v;
        }
    }
    if let Ok(val) = std::env::var("CODE_WEATHER_CLOUDY_COMPLEXITY") {
        if let Ok(v) = val.parse::<u8>() {
            config.thresholds.cloudy_complexity = v;
        }
    }
    if let Ok(val) = std::env::var("CODE_WEATHER_SUNNY_DOCS") {
        if let Ok(v) = val.parse::<u8>() {
            config.thresholds.sunny_docs = v;
        }
    }
    if let Ok(val) = std::env::var("CODE_WEATHER_CLOUDY_DOCS") {
        if let Ok(v) = val.parse::<u8>() {
            config.thresholds.cloudy_docs = v;
        }
    }

    // Analysis overrides
    if let Ok(val) = std::env::var("CODE_WEATHER_SKIP_TESTS") {
        config.analysis.skip_tests = parse_bool(&val);
    }
    if let Ok(val) = std::env::var("CODE_WEATHER_SKIP_GIT") {
        config.analysis.analyze_git = !parse_bool(&val);
    }
    if let Ok(val) = std::env::var("CODE_WEATHER_MAX_FILE_SIZE") {
        if let Ok(v) = val.parse::<usize>() {
            config.analysis.max_file_size = v;
        }
    }
    if let Ok(val) = std::env::var("CODE_WEATHER_GIT_DEPTH") {
        if let Ok(v) = val.parse::<usize>() {
            config.analysis.git_depth = v;
        }
    }

    // Display overrides
    if let Ok(val) = std::env::var("CODE_WEATHER_NO_COLOR") {
        config.display.color = !parse_bool(&val);
    }
    if let Ok(val) = std::env::var("CODE_WEATHER_NO_ASCII") {
        config.display.ascii_art = !parse_bool(&val);
    }
    if let Ok(val) = std::env::var("CODE_WEATHER_TEMP_UNIT") {
        if val == "celsius" || val == "fahrenheit" {
            config.display.temp_unit = val;
        }
    }
}

/// Parse boolean from env var (supports "true", "1", "yes")
fn parse_bool(val: &str) -> bool {
    matches!(val.to_lowercase().as_str(), "true" | "1" | "yes" | "on")
}

/// Merge source config into target (source values override target)
fn merge_config(target: &mut Config, source: &Config) {
    // For simplicity, we replace entire sections if they differ from defaults
    // A more sophisticated merge would check field-by-field

    // Thresholds - always override if not default
    let default_thresholds = ThresholdConfig::default();
    if source.thresholds.sunny_coverage != default_thresholds.sunny_coverage {
        target.thresholds.sunny_coverage = source.thresholds.sunny_coverage;
    }
    if source.thresholds.cloudy_coverage != default_thresholds.cloudy_coverage {
        target.thresholds.cloudy_coverage = source.thresholds.cloudy_coverage;
    }
    if source.thresholds.sunny_complexity != default_thresholds.sunny_complexity {
        target.thresholds.sunny_complexity = source.thresholds.sunny_complexity;
    }
    if source.thresholds.cloudy_complexity != default_thresholds.cloudy_complexity {
        target.thresholds.cloudy_complexity = source.thresholds.cloudy_complexity;
    }
    if source.thresholds.sunny_docs != default_thresholds.sunny_docs {
        target.thresholds.sunny_docs = source.thresholds.sunny_docs;
    }
    if source.thresholds.cloudy_docs != default_thresholds.cloudy_docs {
        target.thresholds.cloudy_docs = source.thresholds.cloudy_docs;
    }
    if source.thresholds.sunny_fn_length != default_thresholds.sunny_fn_length {
        target.thresholds.sunny_fn_length = source.thresholds.sunny_fn_length;
    }
    if source.thresholds.cloudy_fn_length != default_thresholds.cloudy_fn_length {
        target.thresholds.cloudy_fn_length = source.thresholds.cloudy_fn_length;
    }

    // Analysis
    let default_analysis = AnalysisConfig::default();
    if source.analysis.exclude != default_analysis.exclude {
        target.analysis.exclude = source.analysis.exclude.clone();
    }
    if source.analysis.include != default_analysis.include {
        target.analysis.include = source.analysis.include.clone();
    }
    if source.analysis.max_file_size != default_analysis.max_file_size {
        target.analysis.max_file_size = source.analysis.max_file_size;
    }
    if source.analysis.analyze_git != default_analysis.analyze_git {
        target.analysis.analyze_git = source.analysis.analyze_git;
    }
    if source.analysis.git_depth != default_analysis.git_depth {
        target.analysis.git_depth = source.analysis.git_depth;
    }
    if source.analysis.skip_tests != default_analysis.skip_tests {
        target.analysis.skip_tests = source.analysis.skip_tests;
    }

    // Display
    let default_display = DisplayConfig::default();
    if source.display.color != default_display.color {
        target.display.color = source.display.color;
    }
    if source.display.ascii_art != default_display.ascii_art {
        target.display.ascii_art = source.display.ascii_art;
    }
    if source.display.temp_unit != default_display.temp_unit {
        target.display.temp_unit = source.display.temp_unit.clone();
    }
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

/// Get user config directory path
pub fn user_config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join(USER_CONFIG_DIR))
}

/// Generate config file content
pub fn generate_config(full: bool) -> String {
    if full {
        format!(
            r#"# Code Weather Configuration
# Full configuration with all options documented.
#
# Config precedence (highest to lowest):
# 1. CLI flags
# 2. Environment variables (CODE_WEATHER_*)
# 3. Project config (.code-weather.toml)
# 4. User config (~/.config/code-weather/config.toml)
# 5. Built-in defaults
#
# Environment variables:
#   CODE_WEATHER_SUNNY_COVERAGE=80
#   CODE_WEATHER_CLOUDY_COVERAGE=50
#   CODE_WEATHER_SKIP_TESTS=true
#   CODE_WEATHER_SKIP_GIT=true
#   CODE_WEATHER_NO_COLOR=true
#   CODE_WEATHER_TEMP_UNIT=celsius

{}
"#,
            toml::to_string_pretty(&Config::default()).unwrap_or_default()
        )
    } else {
        r#"# Code Weather Configuration

[thresholds]
sunny_coverage = 80
cloudy_coverage = 50

[analysis]
exclude = ["node_modules", "vendor", "target", ".git"]
"#
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use tempfile::TempDir;

    // Mutex to prevent env var tests from interfering with each other
    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    fn clear_env_vars() {
        for var in [
            "CODE_WEATHER_SUNNY_COVERAGE",
            "CODE_WEATHER_CLOUDY_COVERAGE",
            "CODE_WEATHER_SUNNY_COMPLEXITY",
            "CODE_WEATHER_CLOUDY_COMPLEXITY",
            "CODE_WEATHER_SUNNY_DOCS",
            "CODE_WEATHER_CLOUDY_DOCS",
            "CODE_WEATHER_SKIP_TESTS",
            "CODE_WEATHER_SKIP_GIT",
            "CODE_WEATHER_MAX_FILE_SIZE",
            "CODE_WEATHER_GIT_DEPTH",
            "CODE_WEATHER_NO_COLOR",
            "CODE_WEATHER_NO_ASCII",
            "CODE_WEATHER_TEMP_UNIT",
        ] {
            std::env::remove_var(var);
        }
    }

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
        assert!(config.contains("CODE_WEATHER_"));
        assert!(config.contains("Config precedence"));
    }

    #[test]
    fn test_load_default() {
        let _lock = ENV_MUTEX.lock().unwrap();
        clear_env_vars();
        let config = load(None).unwrap();
        assert_eq!(config.thresholds.sunny_coverage, 80);
    }

    #[test]
    fn test_env_override_sunny_coverage() {
        let _lock = ENV_MUTEX.lock().unwrap();
        clear_env_vars();
        std::env::set_var("CODE_WEATHER_SUNNY_COVERAGE", "95");

        let config = load(None).unwrap();
        assert_eq!(config.thresholds.sunny_coverage, 95);

        clear_env_vars();
    }

    #[test]
    fn test_env_override_cloudy_coverage() {
        let _lock = ENV_MUTEX.lock().unwrap();
        clear_env_vars();
        std::env::set_var("CODE_WEATHER_CLOUDY_COVERAGE", "65");

        let config = load(None).unwrap();
        assert_eq!(config.thresholds.cloudy_coverage, 65);

        clear_env_vars();
    }

    #[test]
    fn test_env_override_skip_tests() {
        let _lock = ENV_MUTEX.lock().unwrap();
        clear_env_vars();
        std::env::set_var("CODE_WEATHER_SKIP_TESTS", "true");

        let config = load(None).unwrap();
        assert!(config.analysis.skip_tests);

        clear_env_vars();
    }

    #[test]
    fn test_env_override_skip_git() {
        let _lock = ENV_MUTEX.lock().unwrap();
        clear_env_vars();
        std::env::set_var("CODE_WEATHER_SKIP_GIT", "1");

        let config = load(None).unwrap();
        assert!(!config.analysis.analyze_git);

        clear_env_vars();
    }

    #[test]
    fn test_env_override_no_color() {
        let _lock = ENV_MUTEX.lock().unwrap();
        clear_env_vars();
        std::env::set_var("CODE_WEATHER_NO_COLOR", "yes");

        let config = load(None).unwrap();
        assert!(!config.display.color);

        clear_env_vars();
    }

    #[test]
    fn test_env_override_temp_unit() {
        let _lock = ENV_MUTEX.lock().unwrap();
        clear_env_vars();
        std::env::set_var("CODE_WEATHER_TEMP_UNIT", "fahrenheit");

        let config = load(None).unwrap();
        assert_eq!(config.display.temp_unit, "fahrenheit");

        clear_env_vars();
    }

    #[test]
    fn test_parse_bool_variants() {
        assert!(parse_bool("true"));
        assert!(parse_bool("TRUE"));
        assert!(parse_bool("1"));
        assert!(parse_bool("yes"));
        assert!(parse_bool("YES"));
        assert!(parse_bool("on"));
        assert!(!parse_bool("false"));
        assert!(!parse_bool("0"));
        assert!(!parse_bool("no"));
        assert!(!parse_bool("off"));
        assert!(!parse_bool("random"));
    }

    #[test]
    fn test_merge_config_overrides() {
        let mut target = Config::default();
        let mut source = Config::default();
        source.thresholds.sunny_coverage = 95;
        source.analysis.skip_tests = true;

        merge_config(&mut target, &source);

        assert_eq!(target.thresholds.sunny_coverage, 95);
        assert!(target.analysis.skip_tests);
        // Unchanged fields stay at default
        assert_eq!(target.thresholds.cloudy_coverage, 50);
    }

    #[test]
    fn test_user_config_dir() {
        let dir = user_config_dir();
        // Should return Some on most systems
        if let Some(path) = dir {
            assert!(path.to_string_lossy().contains("code-weather"));
        }
    }

    #[test]
    fn test_project_config_overrides_user() {
        let _lock = ENV_MUTEX.lock().unwrap();
        clear_env_vars();

        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join(".code-weather.toml");
        std::fs::write(
            &config_path,
            r#"
[thresholds]
sunny_coverage = 99
"#,
        )
        .unwrap();

        let config = load(Some(&config_path)).unwrap();
        assert_eq!(config.thresholds.sunny_coverage, 99);
    }

    #[test]
    fn test_env_overrides_project() {
        let _lock = ENV_MUTEX.lock().unwrap();
        clear_env_vars();
        std::env::set_var("CODE_WEATHER_SUNNY_COVERAGE", "100");

        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join(".code-weather.toml");
        std::fs::write(
            &config_path,
            r#"
[thresholds]
sunny_coverage = 50
"#,
        )
        .unwrap();

        let config = load(Some(&config_path)).unwrap();
        // Env var wins over project config
        assert_eq!(config.thresholds.sunny_coverage, 100);

        clear_env_vars();
    }
}
