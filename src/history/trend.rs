//! Trend analysis for historical weather data

use super::storage::HistoryEntry;
use chrono::{DateTime, Duration, Utc};

/// Direction of a trend
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrendDirection {
    /// Metrics improving over time
    Improving,
    /// Metrics declining over time
    Declining,
    /// No significant change
    Stable,
}

impl std::fmt::Display for TrendDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Improving => write!(f, "Improving"),
            Self::Declining => write!(f, "Declining"),
            Self::Stable => write!(f, "Stable"),
        }
    }
}

/// A single metric trend
#[derive(Debug, Clone)]
pub struct Trend {
    /// Name of the metric
    pub name: String,
    /// Current value
    pub current: f32,
    /// Previous value (from comparison period)
    pub previous: f32,
    /// Percentage change
    pub change_percent: f32,
    /// Direction of trend
    pub direction: TrendDirection,
}

impl Trend {
    /// Create a new trend from current and previous values
    /// Higher values are considered better (e.g., coverage, visibility)
    pub fn new(name: &str, current: f32, previous: f32) -> Self {
        let change_percent = if previous > 0.0 {
            ((current - previous) / previous) * 100.0
        } else {
            0.0
        };

        let direction = if change_percent > 5.0 {
            TrendDirection::Improving
        } else if change_percent < -5.0 {
            TrendDirection::Declining
        } else {
            TrendDirection::Stable
        };

        Self {
            name: name.to_string(),
            current,
            previous,
            change_percent,
            direction,
        }
    }

    /// Create a trend where lower values are better (e.g., wind speed)
    pub fn new_lower_better(name: &str, current: f32, previous: f32) -> Self {
        let change_percent = if previous > 0.0 {
            ((current - previous) / previous) * 100.0
        } else {
            0.0
        };

        // Invert direction - decreasing is improving
        let direction = if change_percent < -5.0 {
            TrendDirection::Improving
        } else if change_percent > 5.0 {
            TrendDirection::Declining
        } else {
            TrendDirection::Stable
        };

        Self {
            name: name.to_string(),
            current,
            previous,
            change_percent,
            direction,
        }
    }

    /// Check if this trend represents a regression
    pub fn is_regression(&self) -> bool {
        self.direction == TrendDirection::Declining
    }

    /// Format as ASCII chart bar
    pub fn chart_bar(&self, width: usize) -> String {
        let filled = ((self.current / 100.0) * width as f32).round() as usize;
        let filled = filled.min(width);
        let empty = width - filled;

        format!("[{}{}]", "=".repeat(filled), " ".repeat(empty))
    }
}

/// Full trend analysis result
#[derive(Debug, Clone)]
pub struct TrendAnalysis {
    /// Project path
    pub project: String,
    /// Analysis time period (days)
    pub period_days: u32,
    /// Number of data points analyzed
    pub data_points: usize,
    /// Individual metric trends
    pub trends: Vec<Trend>,
    /// Overall direction
    pub overall: TrendDirection,
    /// Whether there are any regressions
    pub has_regressions: bool,
}

impl TrendAnalysis {
    /// Analyze trends from history entries
    pub fn analyze(project: &str, entries: &[&HistoryEntry], period_days: u32) -> Option<Self> {
        if entries.len() < 2 {
            return None;
        }

        let now = Utc::now();
        let midpoint = now - Duration::days(period_days as i64 / 2);

        // Split into recent and older halves
        let (older, recent): (Vec<_>, Vec<_>) =
            entries.iter().partition(|e| e.timestamp < midpoint);

        if older.is_empty() || recent.is_empty() {
            return None;
        }

        // Calculate averages
        let recent_avg = average_metrics(&recent);
        let older_avg = average_metrics(&older);

        // Build trends
        let trends = vec![
            Trend::new("Score", recent_avg.score, older_avg.score),
            Trend::new("Humidity (Coverage)", recent_avg.humidity, older_avg.humidity),
            Trend::new("Visibility (Docs)", recent_avg.visibility, older_avg.visibility),
            Trend::new_lower_better("Wind (Churn)", recent_avg.wind_speed, older_avg.wind_speed),
        ];

        // Determine overall direction
        let improving = trends.iter().filter(|t| t.direction == TrendDirection::Improving).count();
        let declining = trends.iter().filter(|t| t.direction == TrendDirection::Declining).count();

        let overall = if improving > declining {
            TrendDirection::Improving
        } else if declining > improving {
            TrendDirection::Declining
        } else {
            TrendDirection::Stable
        };

        let has_regressions = trends.iter().any(|t| t.is_regression());

        Some(Self {
            project: project.to_string(),
            period_days,
            data_points: entries.len(),
            trends,
            overall,
            has_regressions,
        })
    }

    /// Get regressions for warnings
    pub fn regressions(&self) -> Vec<&Trend> {
        self.trends.iter().filter(|t| t.is_regression()).collect()
    }

    /// Format as ASCII chart
    pub fn ascii_chart(&self) -> String {
        let mut lines = vec![
            format!("Trend Analysis: {} ({} days, {} samples)", 
                self.project, self.period_days, self.data_points),
            format!("Overall: {}", self.overall),
            String::new(),
        ];

        for trend in &self.trends {
            let arrow = match trend.direction {
                TrendDirection::Improving => "^",
                TrendDirection::Declining => "v",
                TrendDirection::Stable => "-",
            };
            let bar = trend.chart_bar(20);
            let change = if trend.change_percent.abs() > 0.1 {
                format!("{:+.1}%", trend.change_percent)
            } else {
                "~0%".to_string()
            };
            lines.push(format!(
                "{:25} {} {:6.1} {} {}",
                trend.name, bar, trend.current, arrow, change
            ));
        }

        if self.has_regressions {
            lines.push(String::new());
            lines.push("REGRESSION WARNINGS:".to_string());
            for reg in self.regressions() {
                lines.push(format!(
                    "  - {} declined {:.1}%",
                    reg.name,
                    reg.change_percent.abs()
                ));
            }
        }

        lines.join("\n")
    }
}

/// Average metrics from a set of entries
struct AvgMetrics {
    score: f32,
    humidity: f32,
    visibility: f32,
    wind_speed: f32,
}

fn average_metrics(entries: &[&&HistoryEntry]) -> AvgMetrics {
    let n = entries.len() as f32;
    if n == 0.0 {
        return AvgMetrics {
            score: 0.0,
            humidity: 0.0,
            visibility: 0.0,
            wind_speed: 0.0,
        };
    }

    let score: f32 = entries.iter().map(|e| e.score as f32).sum::<f32>() / n;
    let humidity: f32 = entries.iter().map(|e| e.humidity as f32).sum::<f32>() / n;
    let visibility: f32 = entries.iter().map(|e| (e.visibility as f32) * 10.0).sum::<f32>() / n;
    let wind_speed: f32 = entries.iter().map(|e| e.wind_speed as f32).sum::<f32>() / n;

    AvgMetrics {
        score,
        humidity,
        visibility,
        wind_speed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn make_entry(days_ago: i64, score: u8, humidity: u8) -> HistoryEntry {
        HistoryEntry {
            timestamp: Utc::now() - Duration::days(days_ago),
            condition: "Cloudy".to_string(),
            temperature: 50,
            humidity,
            wind_speed: 20,
            visibility: 7,
            score,
            commit: None,
        }
    }

    #[test]
    fn test_trend_improving() {
        let trend = Trend::new("Test", 80.0, 60.0);
        assert_eq!(trend.direction, TrendDirection::Improving);
        assert!(trend.change_percent > 30.0);
    }

    #[test]
    fn test_trend_declining() {
        let trend = Trend::new("Test", 40.0, 60.0);
        assert_eq!(trend.direction, TrendDirection::Declining);
        assert!(trend.change_percent < -30.0);
    }

    #[test]
    fn test_trend_stable() {
        let trend = Trend::new("Test", 60.0, 59.0);
        assert_eq!(trend.direction, TrendDirection::Stable);
    }

    #[test]
    fn test_trend_lower_better() {
        // Wind decreasing is improvement
        let trend = Trend::new_lower_better("Wind", 10.0, 30.0);
        assert_eq!(trend.direction, TrendDirection::Improving);

        // Wind increasing is regression
        let trend = Trend::new_lower_better("Wind", 50.0, 20.0);
        assert_eq!(trend.direction, TrendDirection::Declining);
    }

    #[test]
    fn test_trend_is_regression() {
        let declining = Trend::new("Test", 40.0, 80.0);
        assert!(declining.is_regression());

        let improving = Trend::new("Test", 80.0, 40.0);
        assert!(!improving.is_regression());
    }

    #[test]
    fn test_chart_bar() {
        let trend = Trend::new("Test", 50.0, 50.0);
        let bar = trend.chart_bar(10);
        assert_eq!(bar, "[=====     ]");
    }

    #[test]
    fn test_analyze_insufficient_data() {
        let entry = make_entry(0, 70, 75);
        let entries = vec![&entry];
        let result = TrendAnalysis::analyze("test", &entries, 30);
        assert!(result.is_none());
    }

    #[test]
    fn test_analyze_improving() {
        // Older entries with worse metrics
        let old1 = make_entry(25, 50, 50);
        let old2 = make_entry(20, 55, 55);
        // Recent entries with better metrics
        let new1 = make_entry(5, 75, 80);
        let new2 = make_entry(2, 80, 85);

        let entries = vec![&old1, &old2, &new1, &new2];
        let analysis = TrendAnalysis::analyze("test", &entries, 30).unwrap();

        assert_eq!(analysis.overall, TrendDirection::Improving);
        assert!(!analysis.has_regressions);
    }

    #[test]
    fn test_analyze_declining() {
        // Older entries with better metrics
        let old1 = make_entry(25, 80, 85);
        let old2 = make_entry(20, 75, 80);
        // Recent entries with worse metrics
        let new1 = make_entry(5, 50, 50);
        let new2 = make_entry(2, 45, 45);

        let entries = vec![&old1, &old2, &new1, &new2];
        let analysis = TrendAnalysis::analyze("test", &entries, 30).unwrap();

        assert_eq!(analysis.overall, TrendDirection::Declining);
        assert!(analysis.has_regressions);
    }

    #[test]
    fn test_regressions() {
        let old1 = make_entry(25, 80, 85);
        let new1 = make_entry(2, 50, 50);
        let entries = vec![&old1, &new1];

        let analysis = TrendAnalysis::analyze("test", &entries, 30).unwrap();
        let regressions = analysis.regressions();

        assert!(!regressions.is_empty());
    }

    #[test]
    fn test_ascii_chart_output() {
        let old1 = make_entry(25, 70, 75);
        let new1 = make_entry(2, 75, 80);
        let entries = vec![&old1, &new1];

        let analysis = TrendAnalysis::analyze("test", &entries, 30).unwrap();
        let chart = analysis.ascii_chart();

        assert!(chart.contains("Trend Analysis"));
        assert!(chart.contains("Score"));
    }
}
