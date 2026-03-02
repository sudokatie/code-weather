use clap::{Parser, Subcommand, Args as ClapArgs, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "code-weather")]
#[command(about = "Code quality metrics as weather forecasts")]
#[command(version)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Command>,
    
    /// Show detailed metrics
    #[arg(short, long, global = true)]
    pub verbose: bool,
    
    /// Minimal output
    #[arg(short, long, global = true)]
    pub quiet: bool,
    
    /// Disable colors
    #[arg(long, global = true)]
    pub no_color: bool,
    
    /// Config file path
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Generate weather report for a codebase
    Forecast(ForecastArgs),
    
    /// Create .code-weather.toml config file
    Init(InitArgs),
    
    /// Explain what each weather condition means
    Explain(ExplainArgs),
}

#[derive(ClapArgs, Debug, Clone)]
pub struct ForecastArgs {
    /// Path to analyze
    #[arg(default_value = ".")]
    pub path: PathBuf,
    
    /// Output format
    #[arg(short, long, default_value = "terminal")]
    pub format: OutputFormat,
    
    /// Directory depth for regional breakdown
    #[arg(short, long, default_value = "1")]
    pub depth: usize,
    
    /// Include patterns (glob)
    #[arg(long)]
    pub include: Vec<String>,
    
    /// Exclude patterns (glob)
    #[arg(long)]
    pub exclude: Vec<String>,
    
    /// Analyze only specific language
    #[arg(long)]
    pub lang: Option<String>,
    
    /// Skip git analysis
    #[arg(long)]
    pub no_git: bool,
    
    /// Skip test detection
    #[arg(long)]
    pub no_tests: bool,
    
    /// Override sunny threshold (0-100)
    #[arg(long, value_name = "N")]
    pub threshold_sunny: Option<u8>,
    
    /// Override cloudy threshold (0-100)
    #[arg(long, value_name = "N")]
    pub threshold_cloudy: Option<u8>,
}

#[derive(ClapArgs, Debug, Clone)]
pub struct InitArgs {
    /// Generate config with all options documented
    #[arg(long)]
    pub full: bool,
    
    /// Overwrite existing config
    #[arg(short, long)]
    pub force: bool,
}

#[derive(ClapArgs, Debug, Clone)]
pub struct ExplainArgs {
    /// Specific condition to explain (e.g., "sunny", "cloudy")
    pub condition: Option<String>,
    
    /// Show the specific metrics that determine each condition
    #[arg(long)]
    pub metrics: bool,
}

#[derive(ValueEnum, Debug, Clone, Copy, Default, PartialEq)]
pub enum OutputFormat {
    #[default]
    Terminal,
    Json,
    Markdown,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;
    
    #[test]
    fn test_parse_forecast() {
        let args = Args::parse_from(["code-weather", "forecast"]);
        assert!(matches!(args.command, Some(Command::Forecast(_))));
    }
    
    #[test]
    fn test_parse_forecast_with_path() {
        let args = Args::parse_from(["code-weather", "forecast", "./src"]);
        if let Some(Command::Forecast(f)) = args.command {
            assert_eq!(f.path.to_string_lossy(), "./src");
        } else {
            panic!("Expected Forecast command");
        }
    }
    
    #[test]
    fn test_parse_init() {
        let args = Args::parse_from(["code-weather", "init"]);
        assert!(matches!(args.command, Some(Command::Init(_))));
    }
    
    #[test]
    fn test_parse_init_force() {
        let args = Args::parse_from(["code-weather", "init", "--force"]);
        if let Some(Command::Init(i)) = args.command {
            assert!(i.force);
        } else {
            panic!("Expected Init command");
        }
    }
    
    #[test]
    fn test_parse_explain() {
        let args = Args::parse_from(["code-weather", "explain"]);
        assert!(matches!(args.command, Some(Command::Explain(_))));
    }
    
    #[test]
    fn test_parse_explain_condition() {
        let args = Args::parse_from(["code-weather", "explain", "sunny"]);
        if let Some(Command::Explain(e)) = args.command {
            assert_eq!(e.condition, Some("sunny".to_string()));
        } else {
            panic!("Expected Explain command");
        }
    }
    
    #[test]
    fn test_verbose_flag() {
        let args = Args::parse_from(["code-weather", "-v", "forecast"]);
        assert!(args.verbose);
    }
    
    #[test]
    fn test_quiet_flag() {
        let args = Args::parse_from(["code-weather", "-q", "forecast"]);
        assert!(args.quiet);
    }
    
    #[test]
    fn test_no_color_flag() {
        let args = Args::parse_from(["code-weather", "--no-color", "forecast"]);
        assert!(args.no_color);
    }
    
    #[test]
    fn test_config_path() {
        let args = Args::parse_from(["code-weather", "--config", "/path/to/config", "forecast"]);
        assert_eq!(args.config, Some(PathBuf::from("/path/to/config")));
    }
    
    #[test]
    fn test_output_format_json() {
        let args = Args::parse_from(["code-weather", "forecast", "-f", "json"]);
        if let Some(Command::Forecast(f)) = args.command {
            assert_eq!(f.format, OutputFormat::Json);
        } else {
            panic!("Expected Forecast command");
        }
    }
    
    #[test]
    fn test_help() {
        let mut cmd = Args::command();
        let help = cmd.render_help().to_string();
        assert!(help.contains("code-weather"));
        assert!(help.contains("forecast"));
        assert!(help.contains("init"));
        assert!(help.contains("explain"));
    }
    
    #[test]
    fn test_no_tests_flag() {
        let args = Args::parse_from(["code-weather", "forecast", "--no-tests"]);
        if let Some(Command::Forecast(f)) = args.command {
            assert!(f.no_tests);
        } else {
            panic!("Expected Forecast command");
        }
    }
    
    #[test]
    fn test_threshold_sunny() {
        let args = Args::parse_from(["code-weather", "forecast", "--threshold-sunny", "85"]);
        if let Some(Command::Forecast(f)) = args.command {
            assert_eq!(f.threshold_sunny, Some(85));
        } else {
            panic!("Expected Forecast command");
        }
    }
    
    #[test]
    fn test_threshold_cloudy() {
        let args = Args::parse_from(["code-weather", "forecast", "--threshold-cloudy", "60"]);
        if let Some(Command::Forecast(f)) = args.command {
            assert_eq!(f.threshold_cloudy, Some(60));
        } else {
            panic!("Expected Forecast command");
        }
    }
    
    #[test]
    fn test_init_full() {
        let args = Args::parse_from(["code-weather", "init", "--full"]);
        if let Some(Command::Init(i)) = args.command {
            assert!(i.full);
        } else {
            panic!("Expected Init command");
        }
    }
    
    #[test]
    fn test_explain_metrics() {
        let args = Args::parse_from(["code-weather", "explain", "--metrics"]);
        if let Some(Command::Explain(e)) = args.command {
            assert!(e.metrics);
        } else {
            panic!("Expected Explain command");
        }
    }
    
    #[test]
    fn test_explain_condition_with_metrics() {
        let args = Args::parse_from(["code-weather", "explain", "sunny", "--metrics"]);
        if let Some(Command::Explain(e)) = args.command {
            assert_eq!(e.condition, Some("sunny".to_string()));
            assert!(e.metrics);
        } else {
            panic!("Expected Explain command");
        }
    }
}
