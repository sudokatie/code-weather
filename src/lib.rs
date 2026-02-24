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
    calculate_humidity, calculate_temperature, calculate_visibility, calculate_wind,
    WeatherReport,
};

use std::path::Path;

pub fn run(args: Args) -> Result<()> {
    match &args.command {
        Some(Command::Forecast(forecast_args)) => {
            run_forecast(&args, forecast_args)
        }
        Some(Command::Init(init_args)) => {
            run_init(init_args)
        }
        Some(Command::Explain(explain_args)) => {
            run_explain(explain_args)
        }
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

    // Load config
    let config = config::load(args.config.as_deref())?;

    // Run analysis
    let collector = Collector::new(&config, path);
    let analysis = collector.analyze()?;

    // Calculate weather metrics
    let temperature = calculate_temperature(
        analysis.git.commits_7d,
        analysis.git.commits_30d,
        analysis.git.contributors,
        analysis.git.is_abandoned,
    );

    let humidity = calculate_humidity(
        analysis.tests.coverage_percent,
        analysis.tests.test_to_source_ratio,
    );

    let wind = calculate_wind(analysis.churn.churn_percent, analysis.churn.trend);

    let visibility = calculate_visibility(
        analysis.documentation.coverage_percent,
        analysis.documentation.has_readme,
        analysis.documentation.readme_size,
        analysis.documentation.comment_density,
    );

    // Create weather report
    let report = WeatherReport::new(temperature, humidity, wind, visibility);

    // Output
    let path_str = path.to_string_lossy();
    match forecast.format {
        OutputFormat::Terminal => {
            let output = TerminalOutput::new(args.no_color, args.verbose);
            output.render(&report, &path_str)?;
        }
        OutputFormat::Json => {
            let json = JsonReport::from_weather_report(&report, &path_str);
            println!("{}", json.to_json()?);
        }
        OutputFormat::Markdown => {
            let md = MarkdownOutput::render(&report, &path_str);
            println!("{}", md);
        }
    }

    Ok(())
}

fn run_init(init_args: &cli::InitArgs) -> Result<()> {
    let config_path = Path::new(".code-weather.toml");
    
    if config_path.exists() && !init_args.force {
        return Err(Error::ConfigError(
            "Config file already exists. Use --force to overwrite.".to_string()
        ));
    }

    let content = config::generate_config(false);
    std::fs::write(config_path, content)?;
    
    println!("Created .code-weather.toml");
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
        } else {
            println!("Unknown condition: {}", condition_name);
            println!("Valid conditions: sunny, partly-cloudy, cloudy, rainy, stormy, foggy, frozen");
        }
    } else {
        println!("Code Weather Conditions");
        println!("=======================");
        println!();
        for condition in Condition::all() {
            println!("{} {} - {}", condition.icon(), condition, condition.description());
        }
    }

    Ok(())
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
    fn test_explain_all_conditions() {
        let args = Args {
            command: Some(Command::Explain(cli::ExplainArgs {
                condition: None,
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
            })),
            verbose: false,
            quiet: false,
            no_color: true,
            config: None,
        };
        let result = run(args);
        assert!(result.is_ok());
    }
}
