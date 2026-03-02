/// Visibility based on documentation coverage
#[derive(Debug, Clone, Copy)]
pub struct Visibility {
    pub miles: u8,
}

impl Visibility {
    pub fn new(miles: u8) -> Self {
        Self {
            miles: miles.min(10),
        }
    }

    pub fn description(&self) -> &'static str {
        match self.miles {
            10 => "Clear - excellent documentation",
            7..=9 => "Good - well documented",
            4..=6 => "Moderate - partial documentation",
            2..=3 => "Low - poor documentation",
            _ => "Foggy - minimal documentation",
        }
    }

    pub fn category(&self) -> &'static str {
        match self.miles {
            10 => "clear",
            7..=9 => "good",
            4..=6 => "moderate",
            2..=3 => "low",
            _ => "foggy",
        }
    }
}

/// Calculate visibility from documentation metrics
pub fn calculate_visibility(
    doc_coverage: f64,
    has_readme: bool,
    readme_size: usize,
    comment_density: f64,
) -> Visibility {
    // Base score from doc coverage (0-4 points)
    let coverage_score = match doc_coverage as u8 {
        80..=100 => 4,
        60..=79 => 3,
        40..=59 => 2,
        20..=39 => 1,
        _ => 0,
    };

    // README bonus (0-3 points)
    let readme_score = if has_readme {
        match readme_size {
            2000.. => 3,
            500..=1999 => 2,
            1..=499 => 1,
            _ => 0,
        }
    } else {
        0
    };

    // Comment density bonus (0-3 points)
    let density_score = if comment_density >= 0.2 {
        3
    } else if comment_density >= 0.1 {
        2
    } else if comment_density >= 0.05 {
        1
    } else {
        0
    };

    let total = coverage_score + readme_score + density_score;
    Visibility::new(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_docs_clear() {
        let vis = calculate_visibility(90.0, true, 3000, 0.25);
        assert_eq!(vis.miles, 10);
        assert_eq!(vis.category(), "clear");
    }

    #[test]
    fn test_good_docs() {
        let vis = calculate_visibility(70.0, true, 1000, 0.15);
        assert!(vis.miles >= 7);
        assert_eq!(vis.category(), "good");
    }

    #[test]
    fn test_moderate_docs() {
        let vis = calculate_visibility(50.0, true, 200, 0.08);
        assert!(vis.miles >= 4 && vis.miles <= 6);
    }

    #[test]
    fn test_poor_docs() {
        let vis = calculate_visibility(25.0, false, 0, 0.02);
        assert!(vis.miles <= 3);
    }

    #[test]
    fn test_no_docs_foggy() {
        let vis = calculate_visibility(0.0, false, 0, 0.0);
        assert_eq!(vis.miles, 0);
        assert_eq!(vis.category(), "foggy");
    }

    #[test]
    fn test_readme_bonus() {
        let without = calculate_visibility(50.0, false, 0, 0.1);
        let with_small = calculate_visibility(50.0, true, 100, 0.1);
        let with_large = calculate_visibility(50.0, true, 3000, 0.1);

        assert!(with_small.miles > without.miles);
        assert!(with_large.miles > with_small.miles);
    }
}
