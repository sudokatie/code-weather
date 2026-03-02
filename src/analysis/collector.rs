use crate::analysis::{
    analyze_complexity, analyze_documentation, analyze_structure, analyze_tests, check_readme,
    ComplexityMetrics, DocumentationMetrics, StructureMetrics, TestMetrics,
};
use crate::config::Config;
use crate::error::Error;
use crate::git::{analyze_churn, analyze_git, ChurnMetrics, GitMetrics};
use crate::languages::Language;
use ignore::WalkBuilder;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

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

/// Single file analysis result for parallel processing
#[derive(Debug, Clone, Default)]
struct FileAnalysis {
    complexity: ComplexityMetrics,
    docs: DocumentationMetrics,
    structure: StructureMetrics,
}

/// Collector that runs all analysis
pub struct Collector<'a> {
    config: &'a Config,
    path: &'a Path,
    show_progress: bool,
}

impl<'a> Collector<'a> {
    pub fn new(config: &'a Config, path: &'a Path) -> Self {
        Self {
            config,
            path,
            show_progress: true,
        }
    }

    pub fn with_progress(mut self, show: bool) -> Self {
        self.show_progress = show;
        self
    }

    pub fn analyze(&self) -> Result<AnalysisResult, Error> {
        let start = Instant::now();

        // Collect all source files using ignore crate (respects .gitignore)
        let files: Vec<_> = WalkBuilder::new(self.path)
            .hidden(true) // Skip hidden files
            .git_ignore(true) // Respect .gitignore
            .git_global(true) // Respect global gitignore
            .git_exclude(true) // Respect .git/info/exclude
            .build()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|ft| ft.is_file()).unwrap_or(false))
            .filter(|e| {
                let path_str = e.path().to_string_lossy();
                // Also apply config excludes
                !self
                    .config
                    .analysis
                    .exclude
                    .iter()
                    .any(|ex| path_str.contains(ex))
            })
            .filter(|e| Language::from_path(e.path()).is_some())
            .collect();

        let total_files = files.len();

        // Set up progress bar if we have many files and it's taking time
        let progress = if self.show_progress && total_files > 100 {
            let pb = ProgressBar::new(total_files as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("Analyzing... [{bar:40.cyan/blue}] {pos}/{len} files ({eta})")
                    .unwrap_or_else(|_| ProgressStyle::default_bar())
                    .progress_chars("=>-"),
            );
            Some(pb)
        } else {
            None
        };

        let processed = AtomicUsize::new(0);

        // Process files in parallel using rayon
        let results: Vec<FileAnalysis> = files
            .par_iter()
            .filter_map(|entry| {
                let path = entry.path();

                // Detect language
                let lang = Language::from_path(path)?;

                // Read file (skip if too large or unreadable)
                let content = std::fs::read(path).ok()?;
                if content.len() > self.config.analysis.max_file_size {
                    return None;
                }

                // Analyze
                let complexity = analyze_complexity(&content, lang);
                let docs = analyze_documentation(&content, lang);
                let structure = analyze_structure(&content, lang);

                // Update progress
                let count = processed.fetch_add(1, Ordering::Relaxed) + 1;
                if let Some(ref pb) = progress {
                    pb.set_position(count as u64);
                }

                Some(FileAnalysis {
                    complexity,
                    docs,
                    structure,
                })
            })
            .collect();

        // Finish progress bar
        if let Some(pb) = progress {
            pb.finish_and_clear();
            let elapsed = start.elapsed();
            if elapsed.as_secs() >= 2 {
                eprintln!(
                    "Analyzed {} files in {:.1}s",
                    total_files,
                    elapsed.as_secs_f64()
                );
            }
        }

        // Aggregate results
        let mut total_complexity = ComplexityMetrics::default();
        let mut total_docs = DocumentationMetrics::default();
        let mut total_structure = StructureMetrics::default();

        for analysis in &results {
            total_complexity = merge_complexity(total_complexity, analysis.complexity.clone());
            total_docs = merge_docs(total_docs, analysis.docs.clone());
            total_structure = merge_structure(total_structure, analysis.structure.clone());
        }

        // Check for README
        let (has_readme, readme_size) = check_readme(self.path);
        total_docs.has_readme = has_readme;
        total_docs.readme_size = readme_size;

        // Test metrics
        let tests = analyze_tests(self.path, &self.config.analysis.exclude);

        // Git metrics
        let git = analyze_git(self.path)?;
        let churn = analyze_churn(self.path, 30)?;

        // Save total_lines before moving total_structure
        let total_lines = total_structure.total_lines;

        Ok(AnalysisResult {
            complexity: total_complexity,
            documentation: total_docs,
            structure: total_structure,
            tests,
            git,
            churn,
            file_count: results.len(),
            total_lines,
        })
    }
}

fn merge_complexity(a: ComplexityMetrics, b: ComplexityMetrics) -> ComplexityMetrics {
    let total_funcs = a.total_functions + b.total_functions;
    let total = a.total + b.total;
    ComplexityMetrics {
        total_functions: total_funcs,
        max: a.max.max(b.max),
        min: if a.min == 0 {
            b.min
        } else if b.min == 0 {
            a.min
        } else {
            a.min.min(b.min)
        },
        average: if total_funcs > 0 {
            total as f64 / total_funcs as f64
        } else {
            0.0
        },
        total,
        functions_over_threshold: a.functions_over_threshold + b.functions_over_threshold,
        threshold: a.threshold,
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
        comment_density: if a.comment_density > 0.0 && b.comment_density > 0.0 {
            (a.comment_density + b.comment_density) / 2.0
        } else {
            a.comment_density.max(b.comment_density)
        },
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
        let collector = Collector::new(&config, dir.path()).with_progress(false);
        let result = collector.analyze().unwrap();
        assert_eq!(result.file_count, 0);
    }

    #[test]
    fn test_collector_with_files() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("main.ts"), "function hello() {}").unwrap();

        let config = Config::default();
        let collector = Collector::new(&config, dir.path()).with_progress(false);
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
        let collector = Collector::new(&config, dir.path()).with_progress(false);
        let result = collector.analyze().unwrap();
        assert_eq!(result.file_count, 1);
    }

    #[test]
    fn test_collector_respects_gitignore() {
        use std::process::Command;

        let dir = TempDir::new().unwrap();

        // Initialize git repo (required for ignore crate to use .gitignore)
        Command::new("git")
            .args(["init"])
            .current_dir(dir.path())
            .output()
            .expect("git init failed");

        // Create .gitignore
        std::fs::write(dir.path().join(".gitignore"), "ignored/\n").unwrap();

        // Create ignored directory
        let ignored = dir.path().join("ignored");
        std::fs::create_dir(&ignored).unwrap();
        std::fs::write(ignored.join("skip.ts"), "function skip() {}").unwrap();

        // Create normal file
        std::fs::write(dir.path().join("main.ts"), "function main() {}").unwrap();

        let config = Config::default();
        let collector = Collector::new(&config, dir.path()).with_progress(false);
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
        let collector = Collector::new(&config, dir.path()).with_progress(false);
        let result = collector.analyze().unwrap();
        assert!(result.documentation.has_readme);
    }

    #[test]
    fn test_collector_aggregates_complexity() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("a.ts"), "function a() { if (x) {} }").unwrap();
        std::fs::write(dir.path().join("b.ts"), "function b() {}").unwrap();

        let config = Config::default();
        let collector = Collector::new(&config, dir.path()).with_progress(false);
        let result = collector.analyze().unwrap();
        assert_eq!(result.complexity.total_functions, 2);
    }

    #[test]
    fn test_collector_parallel_processing() {
        let dir = TempDir::new().unwrap();
        // Create multiple files to test parallel processing
        for i in 0..10 {
            std::fs::write(
                dir.path().join(format!("file{}.ts", i)),
                format!("function f{}() {{ if (x) {{}} }}", i),
            )
            .unwrap();
        }

        let config = Config::default();
        let collector = Collector::new(&config, dir.path()).with_progress(false);
        let result = collector.analyze().unwrap();
        assert_eq!(result.file_count, 10);
        assert_eq!(result.complexity.total_functions, 10);
    }
}
