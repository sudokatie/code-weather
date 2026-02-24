use crate::weather::WeatherReport;
use serde::Serialize;

#[derive(Serialize)]
pub struct JsonReport {
    pub path: String,
    pub condition: String,
    pub priority: u8,
    pub temperature: JsonTemperature,
    pub humidity: JsonHumidity,
    pub wind: JsonWind,
    pub visibility: JsonVisibility,
    pub summary: String,
}

#[derive(Serialize)]
pub struct JsonTemperature {
    pub fahrenheit: i32,
    pub celsius: i32,
    pub category: String,
    pub description: String,
}

#[derive(Serialize)]
pub struct JsonHumidity {
    pub percent: u8,
    pub is_estimated: bool,
    pub category: String,
    pub description: String,
}

#[derive(Serialize)]
pub struct JsonWind {
    pub speed: u8,
    pub direction: String,
    pub category: String,
    pub description: String,
}

#[derive(Serialize)]
pub struct JsonVisibility {
    pub miles: u8,
    pub category: String,
    pub description: String,
}

impl JsonReport {
    pub fn from_weather_report(report: &WeatherReport, path: &str) -> Self {
        Self {
            path: path.to_string(),
            condition: format!("{}", report.condition),
            priority: report.condition.priority(),
            temperature: JsonTemperature {
                fahrenheit: report.temperature.fahrenheit,
                celsius: report.temperature.celsius(),
                category: report.temperature.category().to_string(),
                description: report.temperature.description().to_string(),
            },
            humidity: JsonHumidity {
                percent: report.humidity.percent,
                is_estimated: report.humidity.is_estimated,
                category: report.humidity.category().to_string(),
                description: report.humidity.description().to_string(),
            },
            wind: JsonWind {
                speed: report.wind.speed,
                direction: format!("{:?}", report.wind.direction),
                category: report.wind.category().to_string(),
                description: report.wind.description().to_string(),
            },
            visibility: JsonVisibility {
                miles: report.visibility.miles,
                category: report.visibility.category().to_string(),
                description: report.visibility.description().to_string(),
            },
            summary: report.summary(),
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn to_json_compact(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
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
    fn test_json_report_creation() {
        let report = make_report();
        let json = JsonReport::from_weather_report(&report, "./test");
        assert_eq!(json.path, "./test");
        assert_eq!(json.temperature.fahrenheit, 75);
    }

    #[test]
    fn test_json_serialization() {
        let report = make_report();
        let json = JsonReport::from_weather_report(&report, "./test");
        let result = json.to_json();
        assert!(result.is_ok());
        let s = result.unwrap();
        assert!(s.contains("temperature"));
        assert!(s.contains("humidity"));
    }

    #[test]
    fn test_json_compact() {
        let report = make_report();
        let json = JsonReport::from_weather_report(&report, "./test");
        let compact = json.to_json_compact().unwrap();
        let pretty = json.to_json().unwrap();
        assert!(compact.len() < pretty.len());
    }

    #[test]
    fn test_json_condition() {
        let report = make_report();
        let json = JsonReport::from_weather_report(&report, "./test");
        assert!(!json.condition.is_empty());
        assert!(json.priority <= 6);
    }

    #[test]
    fn test_json_humidity_estimated() {
        let mut report = make_report();
        report.humidity.is_estimated = true;
        let json = JsonReport::from_weather_report(&report, "./test");
        assert!(json.humidity.is_estimated);
    }
}
