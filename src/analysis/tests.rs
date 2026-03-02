use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Clone, Default)]
pub struct TestMetrics {
    pub test_files: usize,
    pub source_files: usize,
    pub test_to_source_ratio: f64,
    pub has_coverage_report: bool,
    pub coverage_percent: Option<f64>,
}

pub fn analyze_tests(dir: &Path, exclude: &[String]) -> TestMetrics {
    let mut test_files = 0;
    let mut source_files = 0;

    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();

        // Skip excluded paths
        let path_str = path.to_string_lossy();
        if exclude.iter().any(|ex| path_str.contains(ex)) {
            continue;
        }

        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let filename = path.file_name().and_then(|f| f.to_str()).unwrap_or("");

            if is_source_file(ext) {
                if is_test_file(filename, ext) {
                    test_files += 1;
                } else {
                    source_files += 1;
                }
            }
        }
    }

    let test_to_source_ratio = if source_files > 0 {
        test_files as f64 / source_files as f64
    } else if test_files > 0 {
        1.0 // All tests, no source = 100% coverage estimated
    } else {
        0.0
    };

    let (has_coverage_report, coverage_percent) = find_coverage(dir);

    TestMetrics {
        test_files,
        source_files,
        test_to_source_ratio,
        has_coverage_report,
        coverage_percent,
    }
}

fn is_source_file(ext: &str) -> bool {
    matches!(
        ext.to_lowercase().as_str(),
        "ts" | "tsx" | "js" | "jsx" | "py" | "rs" | "go"
    )
}

fn is_test_file(filename: &str, ext: &str) -> bool {
    let name_lower = filename.to_lowercase();

    // Test file patterns
    match ext.to_lowercase().as_str() {
        "py" => {
            name_lower.starts_with("test_")
                || name_lower.ends_with("_test.py")
                || name_lower == "tests.py"
                || name_lower == "conftest.py"
        }
        "ts" | "tsx" | "js" | "jsx" => {
            name_lower.ends_with(".test.ts")
                || name_lower.ends_with(".test.tsx")
                || name_lower.ends_with(".test.js")
                || name_lower.ends_with(".test.jsx")
                || name_lower.ends_with(".spec.ts")
                || name_lower.ends_with(".spec.tsx")
                || name_lower.ends_with(".spec.js")
                || name_lower.ends_with(".spec.jsx")
        }
        "go" => name_lower.ends_with("_test.go"),
        "rs" => {
            // Rust tests are typically in mod tests or in tests/ dir
            // but we can detect test files by name convention
            name_lower.ends_with("_test.rs") || name_lower.contains("tests")
        }
        _ => false,
    }
}

fn find_coverage(dir: &Path) -> (bool, Option<f64>) {
    // Common coverage file locations
    let coverage_files = [
        ".coverage",
        "coverage.json",
        "coverage/lcov.info",
        "coverage/coverage.json",
        "lcov.info",
        "coverage.lcov",
        "cobertura.xml",
        "coverage.xml",
        "target/coverage/lcov.info", // Rust
    ];

    for file in coverage_files {
        let path = dir.join(file);
        if path.exists() {
            // Try to parse coverage percentage
            if let Some(pct) = parse_coverage(&path) {
                return (true, Some(pct));
            }
            return (true, None);
        }
    }

    (false, None)
}

fn parse_coverage(path: &Path) -> Option<f64> {
    let content = std::fs::read_to_string(path).ok()?;
    let filename = path.file_name()?.to_str()?;

    // Parse lcov format
    if filename.ends_with(".info") || filename == "lcov.info" {
        return parse_lcov(&content);
    }

    // Parse JSON coverage format (simplified)
    if filename.ends_with(".json") {
        return parse_json_coverage(&content);
    }

    None
}

fn parse_lcov(content: &str) -> Option<f64> {
    let mut lines_found: usize = 0;
    let mut lines_hit: usize = 0;

    for line in content.lines() {
        if let Some(val) = line.strip_prefix("LF:") {
            if let Ok(n) = val.trim().parse::<usize>() {
                lines_found += n;
            }
        } else if let Some(val) = line.strip_prefix("LH:") {
            if let Ok(n) = val.trim().parse::<usize>() {
                lines_hit += n;
            }
        }
    }

    if lines_found > 0 {
        Some((lines_hit as f64 / lines_found as f64) * 100.0)
    } else {
        None
    }
}

fn parse_json_coverage(content: &str) -> Option<f64> {
    // Very simplified JSON parsing - look for "pct" or "percent" fields
    // This handles istanbul/nyc format
    if content.contains("\"pct\"") || content.contains("\"percent\"") {
        // Try to find a number after "pct": or "percent":
        for pattern in ["\"pct\":", "\"percent\":"] {
            if let Some(idx) = content.find(pattern) {
                let after = &content[idx + pattern.len()..];
                let num_str: String = after
                    .chars()
                    .skip_while(|c| c.is_whitespace())
                    .take_while(|c| c.is_ascii_digit() || *c == '.')
                    .collect();
                if let Ok(pct) = num_str.parse::<f64>() {
                    return Some(pct);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_detect_python_test_prefix() {
        assert!(is_test_file("test_main.py", "py"));
        assert!(is_test_file("test_utils.py", "py"));
    }

    #[test]
    fn test_detect_python_test_suffix() {
        assert!(is_test_file("main_test.py", "py"));
    }

    #[test]
    fn test_detect_go_test() {
        assert!(is_test_file("main_test.go", "go"));
        assert!(!is_test_file("main.go", "go"));
    }

    #[test]
    fn test_detect_ts_test() {
        assert!(is_test_file("app.test.ts", "ts"));
        assert!(is_test_file("app.spec.ts", "ts"));
        assert!(!is_test_file("app.ts", "ts"));
    }

    #[test]
    fn test_ratio_calculation() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("main.py"), "# source").unwrap();
        std::fs::write(dir.path().join("test_main.py"), "# test").unwrap();

        let metrics = analyze_tests(dir.path(), &[]);
        assert_eq!(metrics.test_files, 1);
        assert_eq!(metrics.source_files, 1);
        assert!((metrics.test_to_source_ratio - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_find_lcov_coverage() {
        let dir = TempDir::new().unwrap();
        let coverage_dir = dir.path().join("coverage");
        std::fs::create_dir(&coverage_dir).unwrap();
        std::fs::write(coverage_dir.join("lcov.info"), "LF:100\nLH:80\n").unwrap();

        let (has_report, pct) = find_coverage(dir.path());
        assert!(has_report);
        assert!((pct.unwrap() - 80.0).abs() < 0.01);
    }

    #[test]
    fn test_find_json_coverage() {
        let dir = TempDir::new().unwrap();
        std::fs::write(
            dir.path().join("coverage.json"),
            r#"{"total": {"lines": {"pct": 75.5}}}"#,
        )
        .unwrap();

        let (has_report, pct) = find_coverage(dir.path());
        assert!(has_report);
        assert!((pct.unwrap() - 75.5).abs() < 0.01);
    }

    #[test]
    fn test_no_coverage_graceful() {
        let dir = TempDir::new().unwrap();
        let (has_report, pct) = find_coverage(dir.path());
        assert!(!has_report);
        assert!(pct.is_none());
    }
}
