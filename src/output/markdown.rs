use crate::weather::WeatherReport;
use crate::{Advisory, AdvisorySeverity, RegionalForecast};

pub struct MarkdownOutput;

impl MarkdownOutput {
    pub fn render(report: &WeatherReport, path: &str) -> String {
        Self::render_full(report, path, &[], &[])
    }

    pub fn render_full(
        report: &WeatherReport,
        path: &str,
        regions: &[RegionalForecast],
        advisories: &[Advisory],
    ) -> String {
        let mut md = String::new();

        md.push_str("# Code Weather Report\n\n");
        md.push_str(&format!("**Path:** `{}`\n\n", path));

        md.push_str(&format!(
            "## {} {}\n\n",
            report.condition.icon(),
            report.condition
        ));
        md.push_str(&format!("{}\n\n", report.condition.description()));

        md.push_str("## Current Conditions\n\n");
        md.push_str("| Metric | Value | Description |\n");
        md.push_str("|--------|-------|-------------|\n");

        md.push_str(&format!(
            "| Temperature | {}°F ({}°C) | {} |\n",
            report.temperature.fahrenheit,
            report.temperature.celsius(),
            report.temperature.description()
        ));

        md.push_str(&format!(
            "| Humidity | {} | {} |\n",
            report.humidity.display(),
            report.humidity.description()
        ));

        md.push_str(&format!(
            "| Wind | {} mph {} | {} |\n",
            report.wind.speed,
            report.wind.direction_description(),
            report.wind.description()
        ));

        md.push_str(&format!(
            "| Visibility | {} miles | {} |\n",
            report.visibility.miles,
            report.visibility.description()
        ));

        // Advisories section
        if !advisories.is_empty() {
            md.push_str("\n## Advisories\n\n");
            for advisory in advisories {
                let icon = match advisory.severity {
                    AdvisorySeverity::Watch => "⚠️",
                    AdvisorySeverity::Warning => "🚨",
                };
                if let Some(ref region) = advisory.region {
                    md.push_str(&format!(
                        "- {} **{}** [{}]: {}\n",
                        icon, advisory.severity, region, advisory.message
                    ));
                } else {
                    md.push_str(&format!(
                        "- {} **{}**: {}\n",
                        icon, advisory.severity, advisory.message
                    ));
                }
            }
        }

        // Regional breakdown
        if !regions.is_empty() {
            md.push_str("\n## Regional Breakdown\n\n");
            md.push_str("| Region | Condition | Summary |\n");
            md.push_str("|--------|-----------|--------|\n");
            for region in regions {
                md.push_str(&format!(
                    "| `{}` | {} {} | {} |\n",
                    region.path,
                    region.condition.icon(),
                    region.condition,
                    region.summary
                ));
            }
        }

        md.push_str("\n---\n\n");
        md.push_str(&format!("*Summary: {}*\n", report.summary()));

        md
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::weather::{Humidity, Temperature, Visibility, Wind, WindDirection};

    fn make_report() -> WeatherReport {
        WeatherReport::new(
            Temperature::new(75),
            Humidity {
                percent: 80,
                is_estimated: false,
            },
            Wind::new(10, WindDirection::Calm),
            Visibility::new(8),
        )
    }

    #[test]
    fn test_markdown_header() {
        let report = make_report();
        let md = MarkdownOutput::render(&report, "./test");
        assert!(md.contains("# Code Weather Report"));
        assert!(md.contains("**Path:** `./test`"));
    }

    #[test]
    fn test_markdown_condition() {
        let report = make_report();
        let md = MarkdownOutput::render(&report, "./test");
        assert!(md.contains("Partly Cloudy") || md.contains("Sunny"));
    }

    #[test]
    fn test_markdown_table() {
        let report = make_report();
        let md = MarkdownOutput::render(&report, "./test");
        assert!(md.contains("| Metric | Value | Description |"));
        assert!(md.contains("| Temperature |"));
        assert!(md.contains("| Humidity |"));
    }

    #[test]
    fn test_markdown_summary() {
        let report = make_report();
        let md = MarkdownOutput::render(&report, "./test");
        assert!(md.contains("*Summary:"));
    }
}
