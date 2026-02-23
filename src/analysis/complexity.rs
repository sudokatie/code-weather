use crate::languages::Language;
use tree_sitter::{Parser, Tree};

/// Metrics from complexity analysis
#[derive(Debug, Clone, Default)]
pub struct ComplexityMetrics {
    /// Total number of functions analyzed
    pub total_functions: usize,
    /// Maximum cyclomatic complexity
    pub max: u32,
    /// Minimum cyclomatic complexity
    pub min: u32,
    /// Average cyclomatic complexity
    pub average: f64,
    /// Sum of all complexities
    pub total: u32,
    /// Functions with complexity over threshold (default 10)
    pub functions_over_threshold: usize,
    /// Threshold used for over_threshold count
    pub threshold: u32,
}

impl ComplexityMetrics {
    pub fn new(threshold: u32) -> Self {
        Self {
            threshold,
            min: u32::MAX,
            ..Default::default()
        }
    }
    
    pub fn add_function(&mut self, complexity: u32) {
        self.total_functions += 1;
        self.total += complexity;
        self.max = self.max.max(complexity);
        self.min = self.min.min(complexity);
        self.average = self.total as f64 / self.total_functions as f64;
        
        if complexity > self.threshold {
            self.functions_over_threshold += 1;
        }
    }
}

/// Analyze cyclomatic complexity of source code
pub fn analyze_complexity(source: &[u8], language: Language) -> ComplexityMetrics {
    let mut metrics = ComplexityMetrics::new(10);
    
    let mut parser = Parser::new();
    parser.set_language(&language.tree_sitter_language()).ok();
    
    let tree = match parser.parse(source, None) {
        Some(t) => t,
        None => return metrics,
    };
    
    analyze_tree(&tree, source, language, &mut metrics);
    
    // Fix min if no functions found
    if metrics.total_functions == 0 {
        metrics.min = 0;
    }
    
    metrics
}

fn analyze_tree(tree: &Tree, source: &[u8], language: Language, metrics: &mut ComplexityMetrics) {
    let mut cursor = tree.walk();
    
    visit_node(&mut cursor, source, language, metrics);
}

fn visit_node(
    cursor: &mut tree_sitter::TreeCursor,
    source: &[u8],
    language: Language,
    metrics: &mut ComplexityMetrics,
) {
    let node = cursor.node();
    let kind = node.kind();
    
    // Check if this is a function definition
    if is_function_node(kind, language) {
        let complexity = calculate_function_complexity(cursor, source, language);
        metrics.add_function(complexity);
    }
    
    // Visit children
    if cursor.goto_first_child() {
        loop {
            visit_node(cursor, source, language, metrics);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        cursor.goto_parent();
    }
}

fn is_function_node(kind: &str, language: Language) -> bool {
    match language {
        Language::TypeScript | Language::JavaScript => {
            matches!(kind, "function_declaration" | "method_definition" | "arrow_function" | "function_expression")
        }
        Language::Python => {
            matches!(kind, "function_definition")
        }
        Language::Rust => {
            matches!(kind, "function_item")
        }
        Language::Go => {
            matches!(kind, "function_declaration" | "method_declaration")
        }
    }
}

fn calculate_function_complexity(
    cursor: &mut tree_sitter::TreeCursor,
    _source: &[u8],
    language: Language,
) -> u32 {
    // Start with base complexity of 1
    let mut complexity = 1u32;
    
    // Count decision points in the function
    count_decision_points(cursor, language, &mut complexity);
    
    complexity
}

fn count_decision_points(
    cursor: &mut tree_sitter::TreeCursor,
    language: Language,
    complexity: &mut u32,
) {
    let node = cursor.node();
    let kind = node.kind();
    
    // Check if this node adds to complexity
    if is_decision_point(kind, language) {
        *complexity += 1;
    }
    
    // Visit children
    if cursor.goto_first_child() {
        loop {
            count_decision_points(cursor, language, complexity);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        cursor.goto_parent();
    }
}

fn is_decision_point(kind: &str, language: Language) -> bool {
    match language {
        Language::TypeScript | Language::JavaScript => {
            matches!(kind, 
                "if_statement" | "else_clause" |
                "for_statement" | "for_in_statement" | 
                "while_statement" | "do_statement" |
                "switch_case" | "catch_clause" |
                "ternary_expression" | "binary_expression" |
                "logical_expression"
            )
        }
        Language::Python => {
            matches!(kind,
                "if_statement" | "elif_clause" | "else_clause" |
                "for_statement" | "while_statement" |
                "except_clause" | "with_statement" |
                "conditional_expression" | "boolean_operator"
            )
        }
        Language::Rust => {
            matches!(kind,
                "if_expression" | "else_clause" |
                "for_expression" | "while_expression" | "loop_expression" |
                "match_arm" | "binary_expression"
            )
        }
        Language::Go => {
            matches!(kind,
                "if_statement" | "else_clause" |
                "for_statement" | "switch_statement" |
                "case_clause" | "select_statement" |
                "binary_expression"
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_empty_file() {
        let metrics = analyze_complexity(b"", Language::TypeScript);
        assert_eq!(metrics.total_functions, 0);
    }
    
    #[test]
    fn test_simple_function_ts() {
        let code = b"function hello() { return 'hello'; }";
        let metrics = analyze_complexity(code, Language::TypeScript);
        assert_eq!(metrics.total_functions, 1);
        assert_eq!(metrics.max, 1);
    }
    
    #[test]
    fn test_function_with_if() {
        let code = b"function check(x) { if (x > 0) return true; return false; }";
        let metrics = analyze_complexity(code, Language::TypeScript);
        assert_eq!(metrics.total_functions, 1);
        assert!(metrics.max >= 2);
    }
    
    #[test]
    fn test_function_with_loop() {
        let code = b"function sum(arr) { let s = 0; for (let x of arr) s += x; return s; }";
        let metrics = analyze_complexity(code, Language::TypeScript);
        assert_eq!(metrics.total_functions, 1);
        assert!(metrics.max >= 2);
    }
    
    #[test]
    fn test_multiple_functions() {
        let code = b"function a() {} function b() { if (x) {} }";
        let metrics = analyze_complexity(code, Language::TypeScript);
        assert_eq!(metrics.total_functions, 2);
    }
    
    #[test]
    fn test_python_function() {
        let code = b"def hello():\n    return 'hello'";
        let metrics = analyze_complexity(code, Language::Python);
        assert_eq!(metrics.total_functions, 1);
    }
    
    #[test]
    fn test_python_function_with_if() {
        let code = b"def check(x):\n    if x > 0:\n        return True\n    return False";
        let metrics = analyze_complexity(code, Language::Python);
        assert_eq!(metrics.total_functions, 1);
        assert!(metrics.max >= 2);
    }
    
    #[test]
    fn test_rust_function() {
        let code = b"fn hello() -> String { String::from(\"hello\") }";
        let metrics = analyze_complexity(code, Language::Rust);
        assert_eq!(metrics.total_functions, 1);
    }
    
    #[test]
    fn test_go_function() {
        let code = b"package main\n\nfunc hello() string { return \"hello\" }";
        let metrics = analyze_complexity(code, Language::Go);
        assert_eq!(metrics.total_functions, 1);
    }
    
    #[test]
    fn test_arrow_function() {
        let code = b"const fn = (x) => x + 1;";
        let metrics = analyze_complexity(code, Language::TypeScript);
        assert_eq!(metrics.total_functions, 1);
    }
    
    #[test]
    fn test_metrics_average() {
        let mut metrics = ComplexityMetrics::new(10);
        metrics.add_function(2);
        metrics.add_function(4);
        assert_eq!(metrics.total_functions, 2);
        assert_eq!(metrics.average, 3.0);
    }
    
    #[test]
    fn test_metrics_min_max() {
        let mut metrics = ComplexityMetrics::new(10);
        metrics.add_function(5);
        metrics.add_function(2);
        metrics.add_function(8);
        assert_eq!(metrics.min, 2);
        assert_eq!(metrics.max, 8);
    }
    
    #[test]
    fn test_over_threshold() {
        let mut metrics = ComplexityMetrics::new(5);
        metrics.add_function(3);  // under
        metrics.add_function(6);  // over
        metrics.add_function(10); // over
        assert_eq!(metrics.functions_over_threshold, 2);
    }
}
