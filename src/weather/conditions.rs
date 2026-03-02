use crossterm::style::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Condition {
    Sunny,
    PartlyCloudy,
    Cloudy,
    Rainy,
    Stormy,
    Foggy,
    Frozen,
}

impl Condition {
    /// Priority for condition (higher = worse, per SPECS.md Section 3.2)
    /// Order: Stormy > Frozen > Rainy > Foggy > Cloudy > PartlyCloudy > Sunny
    pub fn priority(&self) -> u8 {
        match self {
            Self::Sunny => 0,
            Self::PartlyCloudy => 1,
            Self::Cloudy => 2,
            Self::Foggy => 3,
            Self::Rainy => 4,
            Self::Frozen => 5,
            Self::Stormy => 6,
        }
    }

    /// Terminal color for condition
    pub fn color(&self) -> Color {
        match self {
            Self::Sunny => Color::Yellow,
            Self::PartlyCloudy => Color::White,
            Self::Cloudy => Color::DarkGrey,
            Self::Rainy => Color::Blue,
            Self::Stormy => Color::Red,
            Self::Foggy => Color::Grey,
            Self::Frozen => Color::Cyan,
        }
    }

    /// Short description
    pub fn description(&self) -> &'static str {
        match self {
            Self::Sunny => "Clear skies - excellent code health",
            Self::PartlyCloudy => "Mostly clear - good code with minor issues",
            Self::Cloudy => "Overcast - code needs attention",
            Self::Rainy => "Precipitation - several issues detected",
            Self::Stormy => "Severe weather - critical issues present",
            Self::Foggy => "Low visibility - poor documentation coverage",
            Self::Frozen => "Frozen - abandoned or stale codebase",
        }
    }

    /// Icon character
    pub fn icon(&self) -> char {
        match self {
            Self::Sunny => '☀',
            Self::PartlyCloudy => '⛅',
            Self::Cloudy => '☁',
            Self::Rainy => '🌧',
            Self::Stormy => '⛈',
            Self::Foggy => '🌫',
            Self::Frozen => '❄',
        }
    }

    /// ASCII art representation (3 lines)
    pub fn ascii_art(&self) -> [&'static str; 3] {
        match self {
            Self::Sunny => ["   \\   /   ", "    .-.    ", " - (   ) - "],
            Self::PartlyCloudy => ["  \\  /     ", " _/''.-.   ", "   \\_(  ). "],
            Self::Cloudy => ["           ", "    .--.   ", " .-(    ). "],
            Self::Rainy => ["    .-.    ", "   (   ).  ", "  (___(__) "],
            Self::Stormy => ["    .-.    ", "   (   ).  ", "  /(___)\\  "],
            Self::Foggy => ["           ", "_ - _ - _ -", " _ - _ - _ "],
            Self::Frozen => ["    *  *   ", "  *    *   ", "    *  *   "],
        }
    }

    /// All conditions ordered by priority (best to worst)
    pub fn all() -> &'static [Condition] {
        &[
            Self::Sunny,
            Self::PartlyCloudy,
            Self::Cloudy,
            Self::Foggy,
            Self::Rainy,
            Self::Frozen,
            Self::Stormy,
        ]
    }
}

impl std::fmt::Display for Condition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Sunny => "Sunny",
            Self::PartlyCloudy => "Partly Cloudy",
            Self::Cloudy => "Cloudy",
            Self::Rainy => "Rainy",
            Self::Stormy => "Stormy",
            Self::Foggy => "Foggy",
            Self::Frozen => "Frozen",
        };
        write!(f, "{}", name)
    }
}

impl Ord for Condition {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority().cmp(&other.priority())
    }
}

impl PartialOrd for Condition {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sunny_lowest_priority() {
        assert_eq!(Condition::Sunny.priority(), 0);
    }

    #[test]
    fn test_stormy_high_priority() {
        assert!(Condition::Stormy.priority() > Condition::Sunny.priority());
    }

    #[test]
    fn test_stormy_highest_priority() {
        assert_eq!(Condition::Stormy.priority(), 6);
    }

    #[test]
    fn test_frozen_second_highest_priority() {
        assert_eq!(Condition::Frozen.priority(), 5);
    }

    #[test]
    fn test_each_has_unique_icon() {
        let icons: Vec<char> = Condition::all().iter().map(|c| c.icon()).collect();
        let unique: std::collections::HashSet<_> = icons.iter().collect();
        assert_eq!(icons.len(), unique.len());
    }

    #[test]
    fn test_each_has_color() {
        for condition in Condition::all() {
            let _ = condition.color(); // Just verify it doesn't panic
        }
    }

    #[test]
    fn test_ascii_art_consistent_width() {
        for condition in Condition::all() {
            let art = condition.ascii_art();
            let widths: Vec<usize> = art.iter().map(|l| l.len()).collect();
            assert!(
                widths.iter().all(|&w| w == widths[0]),
                "Inconsistent width for {:?}",
                condition
            );
        }
    }

    #[test]
    fn test_ordering() {
        assert!(Condition::Sunny < Condition::Cloudy);
        assert!(Condition::Cloudy < Condition::Rainy);
        assert!(Condition::Frozen < Condition::Stormy);
        assert!(Condition::Stormy.priority() > Condition::Frozen.priority());
    }
}
