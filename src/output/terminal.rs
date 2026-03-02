use crate::weather::{Condition, WeatherReport};
use crate::{Advisory, AdvisorySeverity, RegionalForecast};
use crossterm::style::{Color, Stylize};
use std::io::{self, Write};

/// Box width for terminal output (per SPECS.md Section 7.1)
const BOX_WIDTH: usize = 64;

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
        let date = chrono::Local::now().format("%B %d, %Y").to_string();

        // Top border
        writeln!(stdout)?;
        writeln!(stdout, "╔{}╗", "═".repeat(BOX_WIDTH - 2))?;

        // Title
        let title = "CODE WEATHER FORECAST";
        let title_pad = (BOX_WIDTH - 2 - title.len()) / 2;
        writeln!(
            stdout,
            "║{}{}{}║",
            " ".repeat(title_pad),
            title,
            " ".repeat(BOX_WIDTH - 2 - title_pad - title.len())
        )?;

        // Subtitle (path and date)
        let subtitle = format!("{} - {}", path, date);
        let max_width = BOX_WIDTH - 4;
        let subtitle_display: String = if subtitle.len() > max_width {
            subtitle.chars().take(max_width - 3).collect::<String>() + "..."
        } else {
            subtitle.clone()
        };
        let sub_len = subtitle_display.len();
        let sub_pad = (BOX_WIDTH - 2 - sub_len) / 2;
        let sub_pad_right = BOX_WIDTH - 2 - sub_pad - sub_len;
        writeln!(
            stdout,
            "║{}{}{}║",
            " ".repeat(sub_pad),
            subtitle_display,
            " ".repeat(sub_pad_right)
        )?;

        // Separator
        writeln!(stdout, "╠{}╣", "═".repeat(BOX_WIDTH - 2))?;

        // Empty line
        writeln!(stdout, "║{}║", " ".repeat(BOX_WIDTH - 2))?;

        // ASCII art and current conditions side by side
        self.render_conditions_section(&mut stdout, report)?;

        // Empty line
        writeln!(stdout, "║{}║", " ".repeat(BOX_WIDTH - 2))?;

        // Regional forecast (if any)
        if !regions.is_empty() {
            writeln!(stdout, "╠{}╣", "═".repeat(BOX_WIDTH - 2))?;
            self.render_regions_box(&mut stdout, regions)?;
        }

        // Advisories (if any)
        if !advisories.is_empty() {
            writeln!(stdout, "╠{}╣", "═".repeat(BOX_WIDTH - 2))?;
            self.render_advisories_box(&mut stdout, advisories)?;
        }

        // Bottom border
        writeln!(stdout, "╚{}╝", "═".repeat(BOX_WIDTH - 2))?;
        writeln!(stdout)?;

        if self.verbose {
            self.render_verbose(&mut stdout, report)?;
        }

        Ok(())
    }

    fn render_conditions_section(&self, w: &mut impl Write, report: &WeatherReport) -> io::Result<()> {
        let art = report.condition.ascii_art();
        let condition_name = format!("CURRENT CONDITIONS: {}", report.condition);
        let temp_line = format!(
            "Temperature: {}°F ({})",
            report.temperature.fahrenheit,
            report.temperature.category()
        );
        let humidity_line = format!(
            "Humidity: {} ({})",
            report.humidity.display(),
            report.humidity.category()
        );
        let wind_line = format!(
            "Wind: {} mph {} ({})",
            report.wind.speed,
            report.wind.direction_description(),
            report.wind.category()
        );
        let visibility_line = format!("Visibility: {}", report.visibility.category());

        // Line 0: Art[0] + condition name
        self.write_box_line(w, art[0], &condition_name, &report.condition)?;

        // Line 1: Art[1] + temperature
        self.write_box_line(w, art[1], &temp_line, &report.condition)?;

        // Line 2: Art[2] + humidity
        self.write_box_line(w, art[2], &humidity_line, &report.condition)?;

        // Line 3: Art[3] + wind
        self.write_box_line(w, art[3], &wind_line, &report.condition)?;

        // Line 4: Art[4] + visibility
        self.write_box_line(w, art[4], &visibility_line, &report.condition)?;

        Ok(())
    }

    fn write_box_line(
        &self,
        w: &mut impl Write,
        art: &str,
        text: &str,
        condition: &Condition,
    ) -> io::Result<()> {
        // Layout: "║  {art}   {text}   ║"
        let art_width = 15; // Fixed width for art section
        let text_start = art_width + 3;
        let available = BOX_WIDTH - 2 - text_start - 1;
        let truncated: String = text.chars().take(available).collect();
        let padding = available - truncated.len();

        if self.no_color {
            writeln!(
                w,
                "║  {:width$}  {}{}║",
                art,
                truncated,
                " ".repeat(padding + 1),
                width = art_width - 2
            )
        } else {
            writeln!(
                w,
                "║  {}  {}{}║",
                art.with(condition.color()),
                truncated,
                " ".repeat(padding + 1)
            )
        }
    }

    fn render_regions_box(&self, w: &mut impl Write, regions: &[RegionalForecast]) -> io::Result<()> {
        // Header
        let header = "  REGIONAL FORECAST";
        writeln!(
            w,
            "║{}{}║",
            header,
            " ".repeat(BOX_WIDTH - 2 - header.len())
        )?;
        writeln!(w, "║{}║", " ".repeat(BOX_WIDTH - 2))?;

        for region in regions {
            let icon = region.condition.icon();
            let content = format!("{}  {:18} {}", icon, region.path, region.summary);
            let content_len = content.chars().count();
            let padding = if content_len + 4 < BOX_WIDTH - 2 {
                BOX_WIDTH - 2 - content_len - 4
            } else {
                1
            };

            if self.no_color {
                writeln!(w, "║  {}{}║", content, " ".repeat(padding))?;
            } else {
                let colored_path = region.path.clone().with(region.condition.color());
                writeln!(
                    w,
                    "║  {}  {}  {}{}║",
                    icon,
                    colored_path,
                    region.summary,
                    " ".repeat(padding.saturating_sub(2))
                )?;
            }
        }

        writeln!(w, "║{}║", " ".repeat(BOX_WIDTH - 2))?;
        Ok(())
    }

    fn render_advisories_box(&self, w: &mut impl Write, advisories: &[Advisory]) -> io::Result<()> {
        // Header
        let header = "  ADVISORIES";
        writeln!(
            w,
            "║{}{}║",
            header,
            " ".repeat(BOX_WIDTH - 2 - header.len())
        )?;
        writeln!(w, "║{}║", " ".repeat(BOX_WIDTH - 2))?;

        for advisory in advisories {
            let (label, icon) = match advisory.severity {
                AdvisorySeverity::Watch => ("Watch", "⚠️"),
                AdvisorySeverity::Warning => ("Warning", "🚨"),
            };

            let region_str = advisory
                .region
                .as_ref()
                .map(|r| format!(" [{}]", r))
                .unwrap_or_default();

            let content = format!("{} {}{}: {}", icon, label, region_str, advisory.message);
            let content_len = content.chars().count();
            let padding = if content_len + 4 < BOX_WIDTH - 2 {
                BOX_WIDTH - 2 - content_len - 4
            } else {
                1
            };

            if self.no_color {
                writeln!(w, "║  {}{}║", content, " ".repeat(padding))?;
            } else {
                let color = match advisory.severity {
                    AdvisorySeverity::Watch => Color::Yellow,
                    AdvisorySeverity::Warning => Color::Red,
                };
                let msg_line = format!("{}{}: {}", label, region_str, advisory.message);
                let prefix = format!("  {} ", icon);
                let total_len = prefix.chars().count() + msg_line.chars().count();
                let pad = if total_len < BOX_WIDTH - 2 {
                    BOX_WIDTH - 2 - total_len
                } else {
                    1
                };
                writeln!(
                    w,
                    "║{}{}{}║",
                    prefix,
                    msg_line.with(color),
                    " ".repeat(pad)
                )?;
            }
        }

        writeln!(w, "║{}║", " ".repeat(BOX_WIDTH - 2))?;
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
            let mut report = make_report();
            report.condition = *condition;
            // Should not panic
            let _ = output.render(&report, "./test");
        }
    }

    #[test]
    fn test_box_width_constant() {
        assert_eq!(BOX_WIDTH, 64);
    }
}
