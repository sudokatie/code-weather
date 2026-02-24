use crate::weather::WeatherReport;

pub struct MarkdownOutput;

impl MarkdownOutput {
    pub fn render(report: &WeatherReport, path: &str) -> String {
        let mut md = String::new();

        md.push_str("# Code Weather Report\n\n");
        md.push_str(&format!("**Path:** `{}`\n\n", path));

        md.push_str(&format!("## {} {}\n\n", report.condition.icon(), report.condition));
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

        md.push_str("\n---\n\n");
        md.push_str(&format!("*Summary: {}*\n", report.summary()));

        md
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::weather::{Temperature, Humidity, Wind, WindDirection, Visibility};

    fn make_report() -> WeatherReport {
        WeatherReport::new(
            Temperature::new(75),
            Humidity { percent: 80, is_estimated: false },
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
