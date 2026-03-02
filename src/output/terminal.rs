use crate::weather::{WeatherReport, Condition};
use crate::{Advisory, AdvisorySeverity, RegionalForecast};
use crossterm::style::{Color, Stylize};
use std::io::{self, Write};

pub struct TerminalOutput {
    pub no_color: bool,
    pub verbose: bool,
}

impl TerminalOutput {
    pub fn new(no_color: bool, verbose: bool) -> Self {
        Self { no_color, verbose }
    }

    pub fn render(&self, report: &WeatherReport, path: &str) -> io::Result<()> {
        self.render_full(report, path, &[], &[])
    }

    pub fn render_full(
        &self,
        report: &WeatherReport,
        path: &str,
        regions: &[RegionalForecast],
        advisories: &[Advisory],
    ) -> io::Result<()> {
        let mut stdout = io::stdout();

        // Header
        writeln!(stdout)?;
        writeln!(stdout, "  Weather Report for: {}", path)?;
        writeln!(stdout, "  {}", "─".repeat(40))?;
        writeln!(stdout)?;

        // Condition with ASCII art
        self.render_condition(&mut stdout, &report.condition)?;
        writeln!(stdout)?;

        // Main stats
        self.render_stats(&mut stdout, report)?;
        writeln!(stdout)?;

        // Description
        writeln!(stdout, "  {}", report.condition.description())?;
        writeln!(stdout)?;

        // Advisories
        if !advisories.is_empty() {
            self.render_advisories(&mut stdout, advisories)?;
            writeln!(stdout)?;
        }

        // Regional breakdown
        if !regions.is_empty() {
            self.render_regions(&mut stdout, regions)?;
            writeln!(stdout)?;
        }

        if self.verbose {
            self.render_verbose(&mut stdout, report)?;
        }

        Ok(())
    }

    fn render_advisories(&self, w: &mut impl Write, advisories: &[Advisory]) -> io::Result<()> {
        writeln!(w, "  Advisories")?;
        writeln!(w, "  {}", "─".repeat(40))?;
        
        for advisory in advisories {
            let severity_str = match advisory.severity {
                AdvisorySeverity::Watch => "⚠️  WATCH",
                AdvisorySeverity::Warning => "🚨 WARNING",
            };
            
            let color = match advisory.severity {
                AdvisorySeverity::Watch => Color::Yellow,
                AdvisorySeverity::Warning => Color::Red,
            };
            
            if self.no_color {
                if let Some(ref region) = advisory.region {
                    writeln!(w, "  {} [{}]: {}", severity_str, region, advisory.message)?;
                } else {
                    writeln!(w, "  {}: {}", severity_str, advisory.message)?;
                }
            } else {
                let styled = format!("{}", severity_str).with(color);
                if let Some(ref region) = advisory.region {
                    writeln!(w, "  {} [{}]: {}", styled, region, advisory.message)?;
                } else {
                    writeln!(w, "  {}: {}", styled, advisory.message)?;
                }
            }
        }
        
        Ok(())
    }

    fn render_regions(&self, w: &mut impl Write, regions: &[RegionalForecast]) -> io::Result<()> {
        writeln!(w, "  Regional Breakdown")?;
        writeln!(w, "  {}", "─".repeat(40))?;
        
        for region in regions {
            let icon = region.condition.icon();
            if self.no_color {
                writeln!(w, "  {} {} - {}", icon, region.path, region.summary)?;
            } else {
                writeln!(w, "  {} {} - {}", 
                    icon,
                    region.path.clone().with(region.condition.color()),
                    region.summary)?;
            }
        }
        
        Ok(())
    }

    fn render_condition(&self, w: &mut impl Write, condition: &Condition) -> io::Result<()> {
        let art = condition.ascii_art();
        let name = format!("{}", condition);
        let icon = condition.icon();

        for line in &art {
            if self.no_color {
                writeln!(w, "  {}", line)?;
            } else {
                writeln!(w, "  {}", line.with(condition.color()))?;
            }
        }

        if self.no_color {
            writeln!(w, "  {} {}", icon, name)?;
        } else {
            writeln!(w, "  {} {}", icon, name.with(condition.color()).bold())?;
        }

        Ok(())
    }

    fn render_stats(&self, w: &mut impl Write, report: &WeatherReport) -> io::Result<()> {
        let temp = format!("{}°F ({}°C)", report.temperature.fahrenheit, report.temperature.celsius());
        let humidity = report.humidity.display();
        let wind = format!("{} mph {}", report.wind.speed, report.wind.direction_description());
        let visibility = format!("{} miles", report.visibility.miles);

        writeln!(w, "  Temperature:  {}", temp)?;
        writeln!(w, "  Humidity:     {}", humidity)?;
        writeln!(w, "  Wind:         {}", wind)?;
        writeln!(w, "  Visibility:   {}", visibility)?;

        Ok(())
    }

    fn render_verbose(&self, w: &mut impl Write, report: &WeatherReport) -> io::Result<()> {
        writeln!(w, "  Detailed Analysis")?;
        writeln!(w, "  {}", "─".repeat(40))?;
        writeln!(w, "  Temperature: {}", report.temperature.description())?;
        writeln!(w, "  Humidity:    {}", report.humidity.description())?;
        writeln!(w, "  Wind:        {}", report.wind.description())?;
        writeln!(w, "  Visibility:  {}", report.visibility.description())?;
        writeln!(w)?;

        Ok(())
    }
}

pub fn colorize(text: &str, color: Color, no_color: bool) -> String {
    if no_color {
        text.to_string()
    } else {
        format!("{}", text.with(color))
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
    fn test_terminal_output_new() {
        let output = TerminalOutput::new(true, false);
        assert!(output.no_color);
        assert!(!output.verbose);
    }

    #[test]
    fn test_render_no_panic() {
        let output = TerminalOutput::new(true, false);
        let report = make_report();
        // Just verify it doesn't panic
        let _ = output.render(&report, "./test");
    }

    #[test]
    fn test_render_verbose_no_panic() {
        let output = TerminalOutput::new(true, true);
        let report = make_report();
        let _ = output.render(&report, "./test");
    }

    #[test]
    fn test_colorize_disabled() {
        let result = colorize("test", Color::Red, true);
        assert_eq!(result, "test");
    }

    #[test]
    fn test_colorize_enabled() {
        let result = colorize("test", Color::Red, false);
        assert!(result.contains("test"));
    }

    #[test]
    fn test_all_conditions_render() {
        let output = TerminalOutput::new(true, false);
        for condition in Condition::all() {
            let mut buf = Vec::new();
            output.render_condition(&mut buf, condition).unwrap();
            assert!(!buf.is_empty());
        }
    }

    #[test]
    fn test_stats_render() {
        let output = TerminalOutput::new(true, false);
        let report = make_report();
        let mut buf = Vec::new();
        output.render_stats(&mut buf, &report).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("Temperature"));
        assert!(s.contains("Humidity"));
    }

    #[test]
    fn test_verbose_render() {
        let output = TerminalOutput::new(true, true);
        let report = make_report();
        let mut buf = Vec::new();
        output.render_verbose(&mut buf, &report).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("Detailed Analysis"));
    }
}
