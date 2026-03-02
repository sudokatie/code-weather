pub mod analysis;
pub mod cli;
pub mod config;
pub mod error;
pub mod git;
pub mod languages;
pub mod output;
pub mod weather;

use analysis::Collector;
use cli::{Args, Command, ForecastArgs, OutputFormat};
use error::{Error, Result};
use output::{JsonReport, MarkdownOutput, TerminalOutput};
use weather::{
    calculate_humidity, calculate_temperature, calculate_visibility, calculate_wind, WeatherReport,
};

use std::path::Path;

pub fn run(args: Args) -> Result<()> {
    match &args.command {
        Some(Command::Forecast(forecast_args)) => run_forecast(&args, forecast_args),
        Some(Command::Init(init_args)) => run_init(init_args),
        Some(Command::Explain(explain_args)) => run_explain(explain_args),
        None => {
            // Default to forecast in current directory
            let forecast_args = ForecastArgs {
                path: ".".into(),
                format: OutputFormat::Terminal,
                depth: 1,
                include: vec![],
                exclude: vec![],
                lang: None,
                no_git: false,
                no_tests: false,
                threshold_sunny: None,
                threshold_cloudy: None,
            };
            run_forecast(&args, &forecast_args)
        }
    }
}

fn run_forecast(args: &Args, forecast: &cli::ForecastArgs) -> Result<()> {
    let path = &forecast.path;

    // Validate path
    if !path.exists() {
        return Err(Error::FileNotFound(path.clone()));
    }

    // Load config and apply CLI overrides
    let mut config = config::load(args.config.as_deref())?;
    if let Some(sunny) = forecast.threshold_sunny {
        config.thresholds.sunny_coverage = sunny;
    }
    if let Some(cloudy) = forecast.threshold_cloudy {
        config.thresholds.cloudy_coverage = cloudy;
    }
    config.analysis.skip_tests = forecast.no_tests;

    // Run analysis with CLI filters
    let collector = Collector::new(&config, path)
        .with_include(forecast.include.clone())
        .with_exclude(forecast.exclude.clone())
        .with_lang(forecast.lang.clone());
    let analysis = collector.analyze()?;

    // Calculate weather metrics
    let temperature = calculate_temperature(
        analysis.git.commits_7d,
        analysis.git.commits_30d,
        analysis.git.contributors,
        analysis.git.is_abandoned,
    );

    // If tests skipped, use default humidity
    let humidity = if forecast.no_tests {
        weather::Humidity::new(50.0, true) // Neutral humidity when tests skipped
    } else {
        calculate_humidity(
            analysis.tests.coverage_percent,
            analysis.tests.test_to_source_ratio,
        )
    };

    let wind = calculate_wind(analysis.churn.churn_percent, analysis.churn.trend);

    let visibility = calculate_visibility(
        analysis.documentation.coverage_percent,
        analysis.documentation.has_readme,
        analysis.documentation.readme_size,
        analysis.documentation.comment_density,
    );

    // Create weather report with thresholds
    let report = WeatherReport::new_with_thresholds(
        temperature,
        humidity,
        wind,
        visibility,
        config.thresholds.sunny_coverage,
        config.thresholds.cloudy_coverage,
    );

    // Generate advisories
    let advisories = generate_advisories(&analysis, &report);

    // Analyze regions if depth > 0
    let regions = if forecast.depth > 0 {
        analyze_regions(path, &config, forecast.depth)?
    } else {
        vec![]
    };

    // Output
    let path_str = path.to_string_lossy();
    match forecast.format {
        OutputFormat::Terminal => {
            let output = TerminalOutput::new(args.no_color, args.verbose);
            output.render_full(&report, &path_str, &regions, &advisories)?;
        }
        OutputFormat::Json => {
            let json =
                JsonReport::from_weather_report_full(&report, &path_str, &regions, &advisories);
            println!("{}", json.to_json()?);
        }
        OutputFormat::Markdown => {
            let md = MarkdownOutput::render_full(&report, &path_str, &regions, &advisories);
            println!("{}", md);
        }
    }

    Ok(())
}

/// Advisory message for the report
#[derive(Debug, Clone)]
pub struct Advisory {
    pub severity: AdvisorySeverity,
    pub region: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AdvisorySeverity {
    Watch,   // Storm Watch, Fog Advisory
    Warning, // More severe
}

impl std::fmt::Display for AdvisorySeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Watch => write!(f, "Watch"),
            Self::Warning => write!(f, "Warning"),
        }
    }
}

/// Regional forecast
#[derive(Debug, Clone)]
pub struct RegionalForecast {
    pub path: String,
    pub condition: weather::Condition,
    pub summary: String,
}

fn generate_advisories(
    analysis: &analysis::AnalysisResult,
    _report: &WeatherReport,
) -> Vec<Advisory> {
    let mut advisories = Vec::new();

    // Storm watch for high complexity
    if analysis.complexity.max > 30 {
        advisories.push(Advisory {
            severity: AdvisorySeverity::Watch,
            region: None,
            message: format!(
                "High complexity detected (max CC: {})",
                analysis.complexity.max
            ),
        });
    }

    // Fog advisory for poor documentation
    if analysis.documentation.coverage_percent < 20.0 {
        advisories.push(Advisory {
            severity: AdvisorySeverity::Watch,
            region: None,
            message: format!(
                "Low documentation coverage ({:.0}%)",
                analysis.documentation.coverage_percent
            ),
        });
    }

    // Frozen warning for abandoned
    if analysis.git.is_abandoned {
        advisories.push(Advisory {
            severity: AdvisorySeverity::Warning,
            region: None,
            message: "No recent commits - codebase may be abandoned".to_string(),
        });
    }

    // Storm warning for very high complexity
    if analysis.complexity.max > 50 {
        advisories.push(Advisory {
            severity: AdvisorySeverity::Warning,
            region: None,
            message: format!(
                "Critical complexity (CC: {}) needs refactoring",
                analysis.complexity.max
            ),
        });
    }

    // Low test coverage warning
    if let Some(coverage) = analysis.tests.coverage_percent {
        if coverage < 20.0 {
            advisories.push(Advisory {
                severity: AdvisorySeverity::Watch,
                region: None,
                message: format!("Low test coverage ({:.0}%)", coverage),
            });
        }
    }

    advisories
}

fn analyze_regions(
    base_path: &std::path::Path,
    config: &config::Config,
    _depth: usize,
) -> Result<Vec<RegionalForecast>> {
    use std::collections::HashMap;

    let mut regions: HashMap<String, analysis::AnalysisResult> = HashMap::new();

    // Find subdirectories at depth 1
    for entry in std::fs::read_dir(base_path)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Skip excluded
        if config.analysis.exclude.iter().any(|ex| name.contains(ex)) {
            continue;
        }

        // Analyze this region (no progress bar for sub-regions)
        let collector = Collector::new(config, &path).with_progress(false);
        if let Ok(result) = collector.analyze() {
            if result.file_count > 0 {
                regions.insert(name, result);
            }
        }
    }

    // Convert to RegionalForecast
    let mut forecasts: Vec<RegionalForecast> = regions
        .into_iter()
        .map(|(name, analysis)| {
            let condition = determine_condition(&analysis, config);
            let summary = generate_summary(&analysis, &condition);
            RegionalForecast {
                path: format!("{}/", name),
                condition,
                summary,
            }
        })
        .collect();

    // Sort by condition severity (worst first)
    forecasts.sort_by(|a, b| b.condition.priority().cmp(&a.condition.priority()));

    Ok(forecasts)
}

fn determine_condition(
    analysis: &analysis::AnalysisResult,
    config: &config::Config,
) -> weather::Condition {
    use weather::Condition;

    // Priority order per SPECS.md Section 3.2:
    // 1. Stormy (critical issues always surface)
    // 2. Frozen (abandonment is critical context)
    // 3. Rainy
    // 4. Foggy
    // 5. Cloudy
    // 6. Partly Cloudy
    // 7. Sunny

    // Check for stormy (critical issues) - HIGHEST PRIORITY
    if analysis.complexity.max > 50 {
        return Condition::Stormy;
    }

    // Check for frozen (abandoned)
    if analysis.git.is_abandoned {
        return Condition::Frozen;
    }

    // Check for foggy (poor docs)
    if analysis.documentation.coverage_percent < 20.0 {
        return Condition::Foggy;
    }

    // Calculate score based on various metrics
    let coverage = analysis
        .tests
        .coverage_percent
        .unwrap_or(analysis.tests.test_to_source_ratio * 70.0)
        .min(100.0);

    let complexity_score = if analysis.complexity.average < 10.0 {
        100.0
    } else if analysis.complexity.average < 20.0 {
        70.0
    } else if analysis.complexity.average < 30.0 {
        40.0
    } else {
        20.0
    };

    let doc_score = analysis.documentation.coverage_percent;

    // Weighted average
    let score = (coverage * 0.4 + complexity_score * 0.4 + doc_score * 0.2) as u8;

    if score >= config.thresholds.sunny_coverage {
        Condition::Sunny
    } else if score >= config.thresholds.cloudy_coverage {
        Condition::PartlyCloudy
    } else if score >= 30 {
        Condition::Cloudy
    } else {
        Condition::Rainy
    }
}

fn generate_summary(analysis: &analysis::AnalysisResult, condition: &weather::Condition) -> String {
    use weather::Condition;

    match condition {
        Condition::Sunny => "Clean, well-tested".to_string(),
        Condition::PartlyCloudy => "Good shape, minor issues".to_string(),
        Condition::Cloudy => format!(
            "Moderate complexity (avg: {:.1})",
            analysis.complexity.average
        ),
        Condition::Rainy => {
            if let Some(cov) = analysis.tests.coverage_percent {
                format!("Low coverage ({:.0}%)", cov)
            } else {
                "Needs attention".to_string()
            }
        }
        Condition::Stormy => format!("Critical complexity (CC: {})", analysis.complexity.max),
        Condition::Foggy => format!(
            "Poor documentation ({:.0}%)",
            analysis.documentation.coverage_percent
        ),
        Condition::Frozen => "Abandoned".to_string(),
    }
}

fn run_init(init_args: &cli::InitArgs) -> Result<()> {
    let config_path = Path::new(".code-weather.toml");

    if config_path.exists() && !init_args.force {
        return Err(Error::ConfigError(
            "Config file already exists. Use --force to overwrite.".to_string(),
        ));
    }

    let content = config::generate_config(init_args.full);
    std::fs::write(config_path, content)?;

    if init_args.full {
        println!("Created .code-weather.toml (with all options documented)");
    } else {
        println!("Created .code-weather.toml");
    }
    Ok(())
}

fn run_explain(explain_args: &cli::ExplainArgs) -> Result<()> {
    use weather::Condition;

    if let Some(ref condition_name) = explain_args.condition {
        let condition = match condition_name.to_lowercase().as_str() {
            "sunny" => Some(Condition::Sunny),
            "partly-cloudy" | "partlycloudy" | "partly_cloudy" => Some(Condition::PartlyCloudy),
            "cloudy" => Some(Condition::Cloudy),
            "rainy" => Some(Condition::Rainy),
            "stormy" => Some(Condition::Stormy),
            "foggy" => Some(Condition::Foggy),
            "frozen" => Some(Condition::Frozen),
            _ => None,
        };

        if let Some(c) = condition {
            println!("{} {}", c.icon(), c);
            println!();
            println!("{}", c.description());

            if explain_args.metrics {
                println!();
                println!("Metrics that trigger this condition:");
                println!("{}", condition_metrics(&c));
            }
        } else {
            println!("Unknown condition: {}", condition_name);
            println!(
                "Valid conditions: sunny, partly-cloudy, cloudy, rainy, stormy, foggy, frozen"
            );
        }
    } else {
        println!("Code Weather Conditions");
        println!("=======================");
        println!();
        for condition in Condition::all() {
            println!(
                "{} {} - {}",
                condition.icon(),
                condition,
                condition.description()
            );
            if explain_args.metrics {
                println!("   {}", condition_metrics(condition));
                println!();
            }
        }
    }

    Ok(())
}

/// Get metric thresholds for a condition
fn condition_metrics(condition: &weather::Condition) -> &'static str {
    use weather::Condition;
    match condition {
        Condition::Sunny => "Coverage > 80%, Complexity < 10, Docs > 70%, Active commits",
        Condition::PartlyCloudy => "Coverage 50-80%, Complexity 10-20, Docs 50-70%",
        Condition::Cloudy => "Coverage 30-50%, Complexity 20-30, Docs 30-50%",
        Condition::Rainy => "Coverage < 30%, Complexity 30-50, Flaky tests detected",
        Condition::Stormy => "Critical issues, Complexity > 50, Security concerns",
        Condition::Foggy => "Docs < 20%, Poor naming, Deep nesting",
        Condition::Frozen => "No commits 6+ months, Deprecated dependencies",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_forecast_missing_path() {
        let args = Args {
            command: Some(Command::Forecast(cli::ForecastArgs {
                path: "/nonexistent/path".into(),
                format: OutputFormat::Terminal,
                depth: 1,
                include: vec![],
                exclude: vec![],
                lang: None,
                no_git: false,
                no_tests: false,
                threshold_sunny: None,
                threshold_cloudy: None,
            })),
            verbose: false,
            quiet: false,
            no_color: true,
            config: None,
        };
        let result = run(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_forecast_empty_dir() {
        let dir = TempDir::new().unwrap();
        let args = Args {
            command: Some(Command::Forecast(cli::ForecastArgs {
                path: dir.path().to_path_buf(),
                format: OutputFormat::Json,
                depth: 1,
                include: vec![],
                exclude: vec![],
                lang: None,
                no_git: false,
                no_tests: false,
                threshold_sunny: None,
                threshold_cloudy: None,
            })),
            verbose: false,
            quiet: false,
            no_color: true,
            config: None,
        };
        let result = run(args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_init_creates_config() {
        let dir = TempDir::new().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let args = Args {
            command: Some(Command::Init(cli::InitArgs {
                full: false,
                force: false,
            })),
            verbose: false,
            quiet: false,
            no_color: true,
            config: None,
        };
        let result = run(args);
        assert!(result.is_ok());
        assert!(dir.path().join(".code-weather.toml").exists());
    }

    #[test]
    fn test_init_full_config() {
        let dir = TempDir::new().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let args = Args {
            command: Some(Command::Init(cli::InitArgs {
                full: true,
                force: false,
            })),
            verbose: false,
            quiet: false,
            no_color: true,
            config: None,
        };
        let result = run(args);
        assert!(result.is_ok());
        let content = std::fs::read_to_string(dir.path().join(".code-weather.toml")).unwrap();
        // Full config should have more content (comments/docs)
        assert!(content.len() > 100);
    }

    #[test]
    fn test_explain_all_conditions() {
        let args = Args {
            command: Some(Command::Explain(cli::ExplainArgs {
                condition: None,
                metrics: false,
            })),
            verbose: false,
            quiet: false,
            no_color: true,
            config: None,
        };
        let result = run(args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_explain_specific_condition() {
        let args = Args {
            command: Some(Command::Explain(cli::ExplainArgs {
                condition: Some("sunny".to_string()),
                metrics: false,
            })),
            verbose: false,
            quiet: false,
            no_color: true,
            config: None,
        };
        let result = run(args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_explain_with_metrics() {
        let args = Args {
            command: Some(Command::Explain(cli::ExplainArgs {
                condition: Some("stormy".to_string()),
                metrics: true,
            })),
            verbose: false,
            quiet: false,
            no_color: true,
            config: None,
        };
        let result = run(args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_condition_metrics_all() {
        use weather::Condition;
        for condition in Condition::all() {
            let metrics = condition_metrics(condition);
            assert!(!metrics.is_empty());
        }
    }
}
