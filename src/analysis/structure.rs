use crate::languages::Language;
use tree_sitter::Parser;

#[derive(Debug, Clone, Default)]
pub struct StructureMetrics {
    pub max_nesting: usize,
    pub avg_function_length: f64,
    pub max_function_length: usize,
    pub avg_params: f64,
    pub max_params: usize,
    pub total_lines: usize,
    pub function_count: usize,
}

pub fn analyze_structure(source: &[u8], lang: Language) -> StructureMetrics {
    let mut parser = Parser::new();
    parser
        .set_language(&lang.tree_sitter_language())
        .expect("Failed to set language");

    let tree = match parser.parse(source, None) {
        Some(t) => t,
        None => return StructureMetrics::default(),
    };

    let mut max_nesting = 0;
    let mut function_lengths: Vec<usize> = Vec::new();
    let mut param_counts: Vec<usize> = Vec::new();

    analyze_structure_node(
        tree.root_node(),
        source,
        lang,
        0,
        &mut max_nesting,
        &mut function_lengths,
        &mut param_counts,
    );

    // Count non-empty lines
    let source_str = std::str::from_utf8(source).unwrap_or("");
    let total_lines = source_str
        .lines()
        .filter(|l| !l.trim().is_empty())
        .count();

    let function_count = function_lengths.len();
    let avg_function_length = if function_count > 0 {
        function_lengths.iter().sum::<usize>() as f64 / function_count as f64
    } else {
        0.0
    };
    let max_function_length = function_lengths.iter().copied().max().unwrap_or(0);

    let avg_params = if !param_counts.is_empty() {
        param_counts.iter().sum::<usize>() as f64 / param_counts.len() as f64
    } else {
        0.0
    };
    let max_params = param_counts.iter().copied().max().unwrap_or(0);

    StructureMetrics {
        max_nesting,
        avg_function_length,
        max_function_length,
        avg_params,
        max_params,
        total_lines,
        function_count,
    }
}

fn analyze_structure_node(
    node: tree_sitter::Node,
    source: &[u8],
    lang: Language,
    current_nesting: usize,
    max_nesting: &mut usize,
    function_lengths: &mut Vec<usize>,
    param_counts: &mut Vec<usize>,
) {
    // Track nesting for control flow
    let new_nesting = if is_nesting_node(&node, lang) {
        let n = current_nesting + 1;
        if n > *max_nesting {
            *max_nesting = n;
        }
        n
    } else {
        current_nesting
    };

    // Track function metrics
    if is_function_node(&node, lang) {
        let length = count_function_lines(&node, source);
        function_lengths.push(length);

        let params = count_parameters(&node, lang);
        param_counts.push(params);
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            analyze_structure_node(
                child,
                source,
                lang,
                new_nesting,
                max_nesting,
                function_lengths,
                param_counts,
            );
        }
    }
}

fn is_nesting_node(node: &tree_sitter::Node, lang: Language) -> bool {
    let kind = node.kind();
    match lang {
        Language::TypeScript | Language::JavaScript => {
            matches!(
                kind,
                "if_statement"
                    | "for_statement"
                    | "for_in_statement"
                    | "while_statement"
                    | "do_statement"
                    | "switch_statement"
                    | "try_statement"
            )
        }
        Language::Python => {
            matches!(
                kind,
                "if_statement"
                    | "for_statement"
                    | "while_statement"
                    | "try_statement"
                    | "with_statement"
            )
        }
        Language::Rust => {
            matches!(
                kind,
                "if_expression"
                    | "loop_expression"
                    | "for_expression"
                    | "while_expression"
                    | "match_expression"
            )
        }
        Language::Go => {
            matches!(
                kind,
                "if_statement"
                    | "for_statement"
                    | "switch_statement"
                    | "select_statement"
            )
        }
    }
}

fn is_function_node(node: &tree_sitter::Node, lang: Language) -> bool {
    let kind = node.kind();
    match lang {
        Language::TypeScript | Language::JavaScript => {
            matches!(
                kind,
                "function_declaration"
                    | "arrow_function"
                    | "function_expression"
                    | "method_definition"
            )
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

fn count_function_lines(node: &tree_sitter::Node, source: &[u8]) -> usize {
    let start = node.start_position().row;
    let end = node.end_position().row;
    let line_count = end - start + 1;

    // Count non-empty lines within the function
    let text = node.utf8_text(source).unwrap_or("");
    text.lines().filter(|l| !l.trim().is_empty()).count().max(line_count)
}

fn count_parameters(node: &tree_sitter::Node, lang: Language) -> usize {
    // Find parameter list child
    let param_kinds = match lang {
        Language::TypeScript | Language::JavaScript => vec!["formal_parameters"],
        Language::Python => vec!["parameters"],
        Language::Rust => vec!["parameters"],
        Language::Go => vec!["parameter_list"],
    };

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if param_kinds.contains(&child.kind()) {
                // Count parameter children (skip punctuation)
                let mut count = 0;
                for j in 0..child.child_count() {
                    if let Some(param) = child.child(j) {
                        let kind = param.kind();
                        // Skip punctuation and empty params
                        if kind != "," && kind != "(" && kind != ")" && param.is_named() {
                            count += 1;
                        }
                    }
                }
                return count;
            }
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_line_function() {
        let code = b"function f() {}";
        let metrics = analyze_structure(code, Language::TypeScript);
        assert_eq!(metrics.function_count, 1);
        assert!(metrics.max_function_length >= 1);
    }

    #[test]
    fn test_multi_line_function() {
        let code = b"function f() {\n  const a = 1;\n  return a;\n}";
        let metrics = analyze_structure(code, Language::TypeScript);
        assert_eq!(metrics.function_count, 1);
        assert!(metrics.max_function_length >= 3);
    }

    #[test]
    fn test_nesting_if_inside_for() {
        let code = b"function f() {\n  for (let i = 0; i < 10; i++) {\n    if (i > 5) {}\n  }\n}";
        let metrics = analyze_structure(code, Language::TypeScript);
        assert!(metrics.max_nesting >= 2);
    }

    #[test]
    fn test_max_nesting_tracked() {
        let code = b"function f() {\n  if (a) {\n    if (b) {\n      if (c) {}\n    }\n  }\n}";
        let metrics = analyze_structure(code, Language::TypeScript);
        assert!(metrics.max_nesting >= 3);
    }

    #[test]
    fn test_zero_params() {
        let code = b"function f() {}";
        let metrics = analyze_structure(code, Language::TypeScript);
        assert_eq!(metrics.max_params, 0);
    }

    #[test]
    fn test_multiple_params() {
        let code = b"function f(a, b, c) {}";
        let metrics = analyze_structure(code, Language::TypeScript);
        assert_eq!(metrics.max_params, 3);
    }

    #[test]
    fn test_average_params() {
        let code = b"function f(a) {}\nfunction g(b, c) {}";
        let metrics = analyze_structure(code, Language::TypeScript);
        assert!((metrics.avg_params - 1.5).abs() < 0.01);
    }

    #[test]
    fn test_total_lines() {
        let code = b"function f() {\n  return 1;\n}\n\nfunction g() {}";
        let metrics = analyze_structure(code, Language::TypeScript);
        assert!(metrics.total_lines >= 4); // Non-empty lines
    }
}
