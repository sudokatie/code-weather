use crate::analysis::{
    ComplexityMetrics, DocumentationMetrics, StructureMetrics, TestMetrics,
    analyze_complexity, analyze_documentation, analyze_structure, analyze_tests, check_readme,
};
use crate::config::Config;
use crate::error::Error;
use crate::git::{GitMetrics, ChurnMetrics, analyze_git, analyze_churn};
use crate::languages::Language;
use std::path::Path;
use walkdir::WalkDir;

/// Aggregated analysis results for a codebase
#[derive(Debug, Clone, Default)]
pub struct AnalysisResult {
    pub complexity: ComplexityMetrics,
    pub documentation: DocumentationMetrics,
    pub structure: StructureMetrics,
    pub tests: TestMetrics,
    pub git: GitMetrics,
    pub churn: ChurnMetrics,
    pub file_count: usize,
    pub total_lines: usize,
}

/// Collector that runs all analysis
pub struct Collector<'a> {
    config: &'a Config,
    path: &'a Path,
}

impl<'a> Collector<'a> {
    pub fn new(config: &'a Config, path: &'a Path) -> Self {
        Self { config, path }
    }

    pub fn analyze(&self) -> Result<AnalysisResult, Error> {
        let mut result = AnalysisResult::default();
        let mut total_complexity = ComplexityMetrics::default();
        let mut total_docs = DocumentationMetrics::default();
        let mut total_structure = StructureMetrics::default();

        // Walk source files
        for entry in WalkDir::new(self.path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            
            // Skip excluded paths
            let path_str = path.to_string_lossy();
            if self.config.analysis.exclude.iter().any(|ex| path_str.contains(ex)) {
                continue;
            }

            // Detect language
            let lang = match Language::from_path(path) {
                Some(l) => l,
                None => continue,
            };

            // Read file
            let content = match std::fs::read(path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            result.file_count += 1;

            // Analyze
            let complexity = analyze_complexity(&content, lang);
            let docs = analyze_documentation(&content, lang);
            let structure = analyze_structure(&content, lang);

            // Aggregate
            total_complexity = merge_complexity(total_complexity, complexity);
            total_docs = merge_docs(total_docs, docs);
            total_structure = merge_structure(total_structure, structure);
        }

        // Check for README
        let (has_readme, readme_size) = check_readme(self.path);
        total_docs.has_readme = has_readme;
        total_docs.readme_size = readme_size;

        // Test metrics
        result.tests = analyze_tests(self.path, &self.config.analysis.exclude);

        // Git metrics
        result.git = analyze_git(self.path)?;
        result.churn = analyze_churn(self.path, 30)?;

        result.complexity = total_complexity;
        result.documentation = total_docs;
        result.total_lines = total_structure.total_lines;
        result.structure = total_structure;

        Ok(result)
    }
}

fn merge_complexity(a: ComplexityMetrics, b: ComplexityMetrics) -> ComplexityMetrics {
    let total_funcs = a.total_functions + b.total_functions;
    let total = a.total + b.total;
    ComplexityMetrics {
        total_functions: total_funcs,
        max: a.max.max(b.max),
        min: if a.min == 0 { b.min } else if b.min == 0 { a.min } else { a.min.min(b.min) },
        average: if total_funcs > 0 { total as f64 / total_funcs as f64 } else { 0.0 },
        total,
        functions_over_threshold: a.functions_over_threshold + b.functions_over_threshold,
        threshold: a.threshold, // Use same threshold
    }
}

fn merge_docs(a: DocumentationMetrics, b: DocumentationMetrics) -> DocumentationMetrics {
    let total_items = a.total_items + b.total_items;
    let documented_items = a.documented_items + b.documented_items;
    DocumentationMetrics {
        coverage_percent: if total_items > 0 {
            (documented_items as f64 / total_items as f64) * 100.0
        } else {
            100.0
        },
        documented_items,
        total_items,
        has_readme: a.has_readme || b.has_readme,
        readme_size: a.readme_size.max(b.readme_size),
        comment_density: (a.comment_density + b.comment_density) / 2.0,
    }
}

fn merge_structure(a: StructureMetrics, b: StructureMetrics) -> StructureMetrics {
    let total_funcs = a.function_count + b.function_count;
    StructureMetrics {
        max_nesting: a.max_nesting.max(b.max_nesting),
        avg_function_length: if total_funcs > 0 {
            ((a.avg_function_length * a.function_count as f64)
                + (b.avg_function_length * b.function_count as f64))
                / total_funcs as f64
        } else {
            0.0
        },
        max_function_length: a.max_function_length.max(b.max_function_length),
        avg_params: if total_funcs > 0 {
            ((a.avg_params * a.function_count as f64) + (b.avg_params * b.function_count as f64))
                / total_funcs as f64
        } else {
            0.0
        },
        max_params: a.max_params.max(b.max_params),
        total_lines: a.total_lines + b.total_lines,
        function_count: total_funcs,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_collector_empty_dir() {
        let dir = TempDir::new().unwrap();
        let config = Config::default();
        let collector = Collector::new(&config, dir.path());
        let result = collector.analyze().unwrap();
        assert_eq!(result.file_count, 0);
    }

    #[test]
    fn test_collector_with_files() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("main.ts"), "function hello() {}").unwrap();
        
        let config = Config::default();
        let collector = Collector::new(&config, dir.path());
        let result = collector.analyze().unwrap();
        assert_eq!(result.file_count, 1);
    }

    #[test]
    fn test_collector_excludes_node_modules() {
        let dir = TempDir::new().unwrap();
        let nm = dir.path().join("node_modules");
        std::fs::create_dir(&nm).unwrap();
        std::fs::write(nm.join("dep.ts"), "function dep() {}").unwrap();
        std::fs::write(dir.path().join("main.ts"), "function main() {}").unwrap();
        
        let config = Config::default();
        let collector = Collector::new(&config, dir.path());
        let result = collector.analyze().unwrap();
        assert_eq!(result.file_count, 1);
    }

    #[test]
    fn test_merge_complexity() {
        let a = ComplexityMetrics {
            total_functions: 2,
            max: 5,
            min: 1,
            average: 3.0,
            total: 6,
            functions_over_threshold: 0,
            threshold: 10,
        };
        let b = ComplexityMetrics {
            total_functions: 2,
            max: 10,
            min: 2,
            average: 6.0,
            total: 12,
            functions_over_threshold: 1,
            threshold: 10,
        };
        let merged = merge_complexity(a, b);
        assert_eq!(merged.total_functions, 4);
        assert_eq!(merged.max, 10);
        assert_eq!(merged.min, 1);
        assert_eq!(merged.functions_over_threshold, 1);
    }

    #[test]
    fn test_merge_docs() {
        let a = DocumentationMetrics {
            documented_items: 5,
            total_items: 10,
            coverage_percent: 50.0,
            has_readme: true,
            readme_size: 1000,
            comment_density: 0.1,
        };
        let b = DocumentationMetrics {
            documented_items: 5,
            total_items: 10,
            coverage_percent: 50.0,
            has_readme: false,
            readme_size: 0,
            comment_density: 0.2,
        };
        let merged = merge_docs(a, b);
        assert_eq!(merged.total_items, 20);
        assert_eq!(merged.documented_items, 10);
        assert!(merged.has_readme);
    }

    #[test]
    fn test_collector_readme_detection() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("README.md"), "# Test").unwrap();
        std::fs::write(dir.path().join("main.ts"), "function main() {}").unwrap();
        
        let config = Config::default();
        let collector = Collector::new(&config, dir.path());
        let result = collector.analyze().unwrap();
        assert!(result.documentation.has_readme);
    }

    #[test]
    fn test_collector_aggregates_complexity() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("a.ts"), "function a() { if (x) {} }").unwrap();
        std::fs::write(dir.path().join("b.ts"), "function b() {}").unwrap();
        
        let config = Config::default();
        let collector = Collector::new(&config, dir.path());
        let result = collector.analyze().unwrap();
        assert_eq!(result.complexity.total_functions, 2);
    }
}
