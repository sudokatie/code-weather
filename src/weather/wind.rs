use crate::git::ChurnTrend;

/// Wind based on code churn
#[derive(Debug, Clone, Copy)]
pub struct Wind {
    pub speed: u8, // mph
    pub direction: WindDirection,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindDirection {
    Growing,   // Code growing
    Shrinking, // Code shrinking
    Churning,  // High activity, no net change
    Calm,      // Little activity
}

impl Wind {
    pub fn new(speed: u8, direction: WindDirection) -> Self {
        Self {
            speed: speed.min(100),
            direction,
        }
    }

    pub fn description(&self) -> &'static str {
        match self.speed {
            0..=5 => "Calm - stable codebase",
            6..=15 => "Light breeze - normal activity",
            16..=30 => "Moderate wind - active development",
            31..=50 => "Strong wind - high churn",
            _ => "Gale force - extreme churn",
        }
    }

    pub fn direction_description(&self) -> &'static str {
        match self.direction {
            WindDirection::Growing => "from the south (growing)",
            WindDirection::Shrinking => "from the north (shrinking)",
            WindDirection::Churning => "swirling (refactoring)",
            WindDirection::Calm => "calm",
        }
    }

    pub fn category(&self) -> &'static str {
        match self.speed {
            0..=5 => "calm",
            6..=15 => "light",
            16..=30 => "moderate",
            31..=50 => "strong",
            _ => "gale",
        }
    }
}

/// Calculate wind from churn metrics
pub fn calculate_wind(churn_percent: f64, trend: ChurnTrend) -> Wind {
    // Map churn percentage to wind speed
    let speed = match churn_percent as u8 {
        0..=5 => 0,
        6..=15 => 10,
        16..=30 => 20,
        31..=50 => 35,
        51..=75 => 50,
        _ => 75,
    };

    let direction = match trend {
        ChurnTrend::Growing => WindDirection::Growing,
        ChurnTrend::Shrinking => WindDirection::Shrinking,
        ChurnTrend::Refactoring => WindDirection::Churning,
        ChurnTrend::Stable => WindDirection::Calm,
    };

    Wind::new(speed, direction)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_churn_calm() {
        let wind = calculate_wind(0.0, ChurnTrend::Stable);
        assert_eq!(wind.speed, 0);
        assert_eq!(wind.category(), "calm");
    }

    #[test]
    fn test_low_churn_light() {
        let wind = calculate_wind(10.0, ChurnTrend::Growing);
        assert_eq!(wind.category(), "light");
    }

    #[test]
    fn test_moderate_churn() {
        let wind = calculate_wind(25.0, ChurnTrend::Growing);
        assert_eq!(wind.category(), "moderate");
    }

    #[test]
    fn test_high_churn_strong() {
        let wind = calculate_wind(40.0, ChurnTrend::Refactoring);
        assert_eq!(wind.category(), "strong");
    }

    #[test]
    fn test_extreme_churn_gale() {
        let wind = calculate_wind(80.0, ChurnTrend::Shrinking);
        assert_eq!(wind.category(), "gale");
    }

    #[test]
    fn test_direction_growing() {
        let wind = calculate_wind(20.0, ChurnTrend::Growing);
        assert_eq!(wind.direction, WindDirection::Growing);
    }

    #[test]
    fn test_direction_shrinking() {
        let wind = calculate_wind(20.0, ChurnTrend::Shrinking);
        assert_eq!(wind.direction, WindDirection::Shrinking);
    }

    #[test]
    fn test_direction_refactoring() {
        let wind = calculate_wind(30.0, ChurnTrend::Refactoring);
        assert_eq!(wind.direction, WindDirection::Churning);
    }
}
