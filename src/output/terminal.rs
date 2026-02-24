use crate::weather::{WeatherReport, Condition};
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

        if self.verbose {
            self.render_verbose(&mut stdout, report)?;
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
