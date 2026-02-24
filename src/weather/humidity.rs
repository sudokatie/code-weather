/// Humidity based on test coverage
#[derive(Debug, Clone, Copy)]
pub struct Humidity {
    pub percent: u8,
    pub is_estimated: bool,
}

impl Humidity {
    pub fn new(percent: f64, is_estimated: bool) -> Self {
        Self {
            percent: (percent.clamp(0.0, 100.0) as u8),
            is_estimated,
        }
    }

    pub fn description(&self) -> &'static str {
        let base = match self.percent {
            80..=100 => "Humid - excellent test coverage",
            60..=79 => "Comfortable - good test coverage",
            40..=59 => "Moderate - acceptable test coverage",
            20..=39 => "Low - poor test coverage",
            _ => "Dry - minimal or no test coverage",
        };
        base
    }

    pub fn category(&self) -> &'static str {
        match self.percent {
            80..=100 => "humid",
            60..=79 => "comfortable",
            40..=59 => "moderate",
            20..=39 => "low",
            _ => "dry",
        }
    }

    pub fn display(&self) -> String {
        let suffix = if self.is_estimated { " (estimated)" } else { "" };
        format!("{}%{}", self.percent, suffix)
    }
}

/// Calculate humidity from test coverage metrics
pub fn calculate_humidity(
    actual_coverage: Option<f64>,
    test_to_source_ratio: f64,
) -> Humidity {
    if let Some(coverage) = actual_coverage {
        return Humidity::new(coverage, false);
    }

    // Estimate from test/source ratio
    // 1:1 ratio ~ 70% coverage estimate
    // 0.5:1 ratio ~ 35% coverage estimate
    let estimated = (test_to_source_ratio * 70.0).min(100.0);
    Humidity::new(estimated, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_high_coverage_humid() {
        let humidity = calculate_humidity(Some(85.0), 0.0);
        assert_eq!(humidity.percent, 85);
        assert_eq!(humidity.category(), "humid");
    }

    #[test]
    fn test_good_coverage_comfortable() {
        let humidity = calculate_humidity(Some(70.0), 0.0);
        assert_eq!(humidity.category(), "comfortable");
    }

    #[test]
    fn test_moderate_coverage() {
        let humidity = calculate_humidity(Some(50.0), 0.0);
        assert_eq!(humidity.category(), "moderate");
    }

    #[test]
    fn test_low_coverage() {
        let humidity = calculate_humidity(Some(25.0), 0.0);
        assert_eq!(humidity.category(), "low");
    }

    #[test]
    fn test_no_coverage_dry() {
        let humidity = calculate_humidity(Some(10.0), 0.0);
        assert_eq!(humidity.category(), "dry");
    }

    #[test]
    fn test_estimated_from_ratio() {
        let humidity = calculate_humidity(None, 1.0);
        assert!(humidity.is_estimated);
        assert_eq!(humidity.percent, 70);
    }

    #[test]
    fn test_display_actual() {
        let humidity = calculate_humidity(Some(80.0), 0.0);
        assert_eq!(humidity.display(), "80%");
    }

    #[test]
    fn test_display_estimated() {
        let humidity = calculate_humidity(None, 0.5);
        assert!(humidity.display().contains("estimated"));
    }
}
