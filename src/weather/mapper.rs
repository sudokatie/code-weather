use super::{Condition, Temperature, Humidity, Wind, Visibility};

/// Full weather report combining all metrics
#[derive(Debug, Clone)]
pub struct WeatherReport {
    pub condition: Condition,
    pub temperature: Temperature,
    pub humidity: Humidity,
    pub wind: Wind,
    pub visibility: Visibility,
}

impl WeatherReport {
    pub fn new(
        temperature: Temperature,
        humidity: Humidity,
        wind: Wind,
        visibility: Visibility,
    ) -> Self {
        let condition = determine_condition(&temperature, &humidity, &wind, &visibility, 80, 50);
        Self {
            condition,
            temperature,
            humidity,
            wind,
            visibility,
        }
    }

    pub fn new_with_thresholds(
        temperature: Temperature,
        humidity: Humidity,
        wind: Wind,
        visibility: Visibility,
        sunny_threshold: u8,
        cloudy_threshold: u8,
    ) -> Self {
        let condition = determine_condition(&temperature, &humidity, &wind, &visibility, sunny_threshold, cloudy_threshold);
        Self {
            condition,
            temperature,
            humidity,
            wind,
            visibility,
        }
    }

    pub fn summary(&self) -> String {
        format!(
            "{} - {}°F, {}% humidity, {} mph wind, {} mile visibility",
            self.condition,
            self.temperature.fahrenheit,
            self.humidity.percent,
            self.wind.speed,
            self.visibility.miles,
        )
    }
}

/// Determine overall condition from individual metrics
fn determine_condition(
    temperature: &Temperature,
    humidity: &Humidity,
    wind: &Wind,
    visibility: &Visibility,
    sunny_threshold: u8,
    cloudy_threshold: u8,
) -> Condition {
    // Frozen takes precedence
    if temperature.fahrenheit < 32 {
        return Condition::Frozen;
    }

    // Foggy if very low visibility
    if visibility.miles <= 2 {
        return Condition::Foggy;
    }

    // Stormy if high churn and low coverage
    if wind.speed >= 40 && humidity.percent < 40 {
        return Condition::Stormy;
    }

    // Rainy if moderate issues
    if (wind.speed >= 25 && humidity.percent < 50) || visibility.miles <= 4 {
        return Condition::Rainy;
    }

    // Calculate overall health score
    let health = ((humidity.percent as u16 + visibility.miles as u16 * 10) / 2) as u8;

    // Cloudy if below cloudy threshold
    if health < cloudy_threshold || humidity.percent < 60 || visibility.miles < 7 || wind.speed > 15 {
        return Condition::Cloudy;
    }

    // Partly cloudy if below sunny threshold
    if health < sunny_threshold || humidity.percent < 75 || visibility.miles < 9 {
        return Condition::PartlyCloudy;
    }

    // Sunny - everything looks good
    Condition::Sunny
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::weather::wind::WindDirection;

    fn make_temp(f: i32) -> Temperature {
        Temperature::new(f)
    }

    fn make_humidity(pct: u8) -> Humidity {
        Humidity { percent: pct, is_estimated: false }
    }

    fn make_wind(speed: u8) -> Wind {
        Wind::new(speed, WindDirection::Calm)
    }

    fn make_vis(miles: u8) -> Visibility {
        Visibility::new(miles)
    }

    #[test]
    fn test_frozen_abandoned() {
        let report = WeatherReport::new(
            make_temp(20),
            make_humidity(80),
            make_wind(5),
            make_vis(10),
        );
        assert_eq!(report.condition, Condition::Frozen);
    }

    #[test]
    fn test_foggy_low_visibility() {
        let report = WeatherReport::new(
            make_temp(70),
            make_humidity(80),
            make_wind(5),
            make_vis(1),
        );
        assert_eq!(report.condition, Condition::Foggy);
    }

    #[test]
    fn test_stormy_high_churn_low_coverage() {
        let report = WeatherReport::new(
            make_temp(70),
            make_humidity(30),
            make_wind(50),
            make_vis(8),
        );
        assert_eq!(report.condition, Condition::Stormy);
    }

    #[test]
    fn test_rainy_moderate_issues() {
        let report = WeatherReport::new(
            make_temp(70),
            make_humidity(45),
            make_wind(30),
            make_vis(6),
        );
        assert_eq!(report.condition, Condition::Rainy);
    }

    #[test]
    fn test_cloudy_some_concerns() {
        let report = WeatherReport::new(
            make_temp(70),
            make_humidity(55),
            make_wind(10),
            make_vis(6),
        );
        assert_eq!(report.condition, Condition::Cloudy);
    }

    #[test]
    fn test_partly_cloudy_minor() {
        let report = WeatherReport::new(
            make_temp(70),
            make_humidity(70),
            make_wind(10),
            make_vis(8),
        );
        assert_eq!(report.condition, Condition::PartlyCloudy);
    }

    #[test]
    fn test_sunny_all_good() {
        let report = WeatherReport::new(
            make_temp(80),
            make_humidity(85),
            make_wind(5),
            make_vis(10),
        );
        assert_eq!(report.condition, Condition::Sunny);
    }

    #[test]
    fn test_summary_format() {
        let report = WeatherReport::new(
            make_temp(75),
            make_humidity(80),
            make_wind(10),
            make_vis(8),
        );
        let summary = report.summary();
        assert!(summary.contains("75°F"));
        assert!(summary.contains("80%"));
    }
}
