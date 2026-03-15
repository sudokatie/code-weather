//! History storage - persist weather reports to JSON file

use crate::error::{Error, Result};
use crate::weather::{Condition, WeatherReport};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A single historical entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// When the report was recorded
    pub timestamp: DateTime<Utc>,
    /// The condition at that time
    pub condition: String,
    /// Temperature in fahrenheit
    pub temperature: i32,
    /// Humidity percentage (0-100)
    pub humidity: u8,
    /// Wind speed (mph)
    pub wind_speed: u8,
    /// Visibility (miles, 0-10)
    pub visibility: u8,
    /// Overall score (0-100)
    pub score: u8,
    /// Git commit hash if available
    pub commit: Option<String>,
}

impl HistoryEntry {
    /// Create from a weather report
    pub fn from_report(report: &WeatherReport, commit: Option<String>) -> Self {
        Self {
            timestamp: Utc::now(),
            condition: report.condition.to_string(),
            temperature: report.temperature.fahrenheit,
            humidity: report.humidity.percent,
            wind_speed: report.wind.speed,
            visibility: report.visibility.miles,
            score: calculate_score(report),
            commit,
        }
    }

    /// Parse condition from stored string
    pub fn condition(&self) -> Condition {
        match self.condition.to_lowercase().as_str() {
            "sunny" => Condition::Sunny,
            "partly cloudy" => Condition::PartlyCloudy,
            "cloudy" => Condition::Cloudy,
            "rainy" => Condition::Rainy,
            "stormy" => Condition::Stormy,
            "foggy" => Condition::Foggy,
            "frozen" => Condition::Frozen,
            _ => Condition::Cloudy,
        }
    }
}

/// Calculate an overall score from the weather report
fn calculate_score(report: &WeatherReport) -> u8 {
    // Weight: humidity 40%, visibility 30%, temperature 20%, wind 10%
    let humidity_score = report.humidity.percent as f32;
    let visibility_score = (report.visibility.miles as f32) * 10.0; // Scale to 0-100
    let temp = report.temperature.fahrenheit as f32;
    let temp_score = if temp > 50.0 {
        (100.0 - (temp - 50.0).min(50.0)).max(0.0)
    } else if temp < 20.0 {
        (temp * 2.0).max(0.0)
    } else {
        80.0
    };
    let wind_score = (100.0 - (report.wind.speed as f32).min(100.0)).max(0.0);

    let score =
        humidity_score * 0.4 + visibility_score * 0.3 + temp_score * 0.2 + wind_score * 0.1;
    score.round() as u8
}

/// History data for a project
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectHistory {
    pub entries: Vec<HistoryEntry>,
}

impl ProjectHistory {
    /// Add an entry, keeping max 365 entries
    pub fn add(&mut self, entry: HistoryEntry) {
        self.entries.push(entry);
        // Keep last 365 entries
        if self.entries.len() > 365 {
            self.entries.remove(0);
        }
    }

    /// Get entries within a time range
    pub fn entries_since(&self, since: DateTime<Utc>) -> Vec<&HistoryEntry> {
        self.entries
            .iter()
            .filter(|e| e.timestamp >= since)
            .collect()
    }

    /// Get the most recent entry
    pub fn latest(&self) -> Option<&HistoryEntry> {
        self.entries.last()
    }
}

/// Store for all project histories
#[derive(Debug)]
pub struct HistoryStore {
    /// Path to the history file
    path: PathBuf,
    /// Project histories keyed by path
    data: HashMap<String, ProjectHistory>,
}

impl HistoryStore {
    /// Default history file location
    pub fn default_path() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("code-weather")
            .join("history.json")
    }

    /// Open or create a history store
    pub fn open(path: Option<&Path>) -> Result<Self> {
        let path = path
            .map(PathBuf::from)
            .unwrap_or_else(Self::default_path);

        let data = if path.exists() {
            let content = std::fs::read_to_string(&path).map_err(|e| {
                Error::ConfigError(format!("Failed to read history: {}", e))
            })?;
            serde_json::from_str(&content).map_err(|e| {
                Error::ConfigError(format!("Failed to parse history: {}", e))
            })?
        } else {
            HashMap::new()
        };

        Ok(Self { path, data })
    }

    /// Record a weather report for a project
    pub fn record(&mut self, project_path: &str, report: &WeatherReport, commit: Option<String>) {
        let entry = HistoryEntry::from_report(report, commit);
        self.data
            .entry(project_path.to_string())
            .or_default()
            .add(entry);
    }

    /// Get history for a project
    pub fn get(&self, project_path: &str) -> Option<&ProjectHistory> {
        self.data.get(project_path)
    }

    /// Save to disk
    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                Error::ConfigError(format!("Failed to create history dir: {}", e))
            })?;
        }

        let content = serde_json::to_string_pretty(&self.data).map_err(|e| {
            Error::ConfigError(format!("Failed to serialize history: {}", e))
        })?;

        std::fs::write(&self.path, content).map_err(|e| {
            Error::ConfigError(format!("Failed to write history: {}", e))
        })?;

        Ok(())
    }

    /// Clear history for a project
    pub fn clear(&mut self, project_path: &str) {
        self.data.remove(project_path);
    }

    /// List all tracked projects
    pub fn projects(&self) -> Vec<&str> {
        self.data.keys().map(|s| s.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::weather::{Humidity, Temperature, Visibility, Wind, WindDirection};
    use tempfile::TempDir;

    fn test_report() -> WeatherReport {
        WeatherReport::new(
            Temperature::new(45),
            Humidity::new(75.0, false),
            Wind::new(15, WindDirection::Calm),
            Visibility::new(8),
        )
    }

    #[test]
    fn test_entry_from_report() {
        let report = test_report();
        let entry = HistoryEntry::from_report(&report, Some("abc123".to_string()));

        assert_eq!(entry.temperature, 45);
        assert_eq!(entry.humidity, 75);
        assert_eq!(entry.commit, Some("abc123".to_string()));
    }

    #[test]
    fn test_calculate_score() {
        let report = test_report();
        let score = calculate_score(&report);
        // With 75% humidity, 8 miles visibility, 45 temp, 15 wind
        // Score should be reasonable
        assert!(score > 50);
        assert!(score < 90);
    }

    #[test]
    fn test_project_history_add() {
        let mut history = ProjectHistory::default();
        let report = test_report();

        for _ in 0..10 {
            history.add(HistoryEntry::from_report(&report, None));
        }

        assert_eq!(history.entries.len(), 10);
    }

    #[test]
    fn test_project_history_limit() {
        let mut history = ProjectHistory::default();
        let report = test_report();

        for _ in 0..400 {
            history.add(HistoryEntry::from_report(&report, None));
        }

        // Should be capped at 365
        assert_eq!(history.entries.len(), 365);
    }

    #[test]
    fn test_store_record_and_get() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("history.json");
        let mut store = HistoryStore::open(Some(&path)).unwrap();

        let report = test_report();
        store.record("/test/project", &report, None);

        let history = store.get("/test/project").unwrap();
        assert_eq!(history.entries.len(), 1);
    }

    #[test]
    fn test_store_save_and_reload() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("history.json");

        // Save
        {
            let mut store = HistoryStore::open(Some(&path)).unwrap();
            let report = test_report();
            store.record("/test/project", &report, None);
            store.save().unwrap();
        }

        // Reload
        {
            let store = HistoryStore::open(Some(&path)).unwrap();
            let history = store.get("/test/project").unwrap();
            assert_eq!(history.entries.len(), 1);
        }
    }

    #[test]
    fn test_entry_condition_parse() {
        let entry = HistoryEntry {
            timestamp: Utc::now(),
            condition: "Stormy".to_string(),
            temperature: 70,
            humidity: 80,
            wind_speed: 50,
            visibility: 3,
            score: 40,
            commit: None,
        };

        assert_eq!(entry.condition(), Condition::Stormy);
    }
}
