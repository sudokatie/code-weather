/// Temperature based on repository activity
#[derive(Debug, Clone, Copy)]
pub struct Temperature {
    pub fahrenheit: i32,
}

impl Temperature {
    pub fn new(fahrenheit: i32) -> Self {
        Self { fahrenheit: fahrenheit.clamp(0, 100) }
    }

    pub fn celsius(&self) -> i32 {
        ((self.fahrenheit - 32) * 5) / 9
    }

    pub fn description(&self) -> &'static str {
        match self.fahrenheit {
            90..=100 => "Hot - very active development",
            70..=89 => "Comfortable - healthy activity",
            50..=69 => "Cool - moderate activity",
            32..=49 => "Cold - low activity",
            _ => "Freezing - abandoned or stale",
        }
    }

    pub fn category(&self) -> &'static str {
        match self.fahrenheit {
            90..=100 => "hot",
            70..=89 => "comfortable",
            50..=69 => "cool",
            32..=49 => "cold",
            _ => "freezing",
        }
    }
}

/// Calculate temperature from git activity metrics
pub fn calculate_temperature(
    commits_7d: usize,
    commits_30d: usize,
    contributors: usize,
    is_abandoned: bool,
) -> Temperature {
    if is_abandoned {
        return Temperature::new(20); // Freezing
    }

    // Base score from recent commits (0-60 points)
    let recent_score = match commits_7d {
        0 => 0,
        1..=2 => 15,
        3..=5 => 30,
        6..=10 => 45,
        _ => 60,
    };

    // Monthly activity bonus (0-20 points)
    let monthly_score = match commits_30d {
        0..=5 => 0,
        6..=15 => 10,
        16..=30 => 15,
        _ => 20,
    };

    // Contributor bonus (0-20 points)
    let contributor_score = match contributors {
        0..=1 => 0,
        2..=3 => 10,
        4..=10 => 15,
        _ => 20,
    };

    let total = recent_score + monthly_score + contributor_score;
    
    // Map 0-100 score to temperature range (32-100°F)
    let temp = 32 + (total * 68 / 100);
    
    Temperature::new(temp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_high_activity_hot() {
        let temp = calculate_temperature(20, 50, 5, false);
        assert!(temp.fahrenheit >= 90);
    }

    #[test]
    fn test_moderate_activity_comfortable() {
        let temp = calculate_temperature(7, 20, 3, false);
        assert!(temp.fahrenheit >= 70);
        assert!(temp.fahrenheit < 90);
    }

    #[test]
    fn test_low_activity_cool() {
        let temp = calculate_temperature(1, 5, 1, false);
        assert!(temp.fahrenheit >= 32);
        assert!(temp.fahrenheit < 70);
    }

    #[test]
    fn test_no_activity_cold() {
        let temp = calculate_temperature(0, 2, 1, false);
        assert!(temp.fahrenheit >= 32);
        assert!(temp.fahrenheit < 50);
    }

    #[test]
    fn test_abandoned_freezing() {
        let temp = calculate_temperature(0, 0, 0, true);
        assert!(temp.fahrenheit < 32);
    }

    #[test]
    fn test_contributor_bonus() {
        let temp_solo = calculate_temperature(5, 15, 1, false);
        let temp_team = calculate_temperature(5, 15, 10, false);
        assert!(temp_team.fahrenheit > temp_solo.fahrenheit);
    }

    #[test]
    fn test_description_matches() {
        let hot = Temperature::new(95);
        assert!(hot.description().contains("Hot"));

        let cold = Temperature::new(35);
        assert!(cold.description().contains("Cold"));

        let freezing = Temperature::new(20);
        assert!(freezing.description().contains("Freezing"));
    }

    #[test]
    fn test_celsius_conversion() {
        let temp = Temperature::new(68);
        assert_eq!(temp.celsius(), 20);

        let freezing = Temperature::new(32);
        assert_eq!(freezing.celsius(), 0);
    }
}
