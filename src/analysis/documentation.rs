use crate::languages::Language;
use tree_sitter::Parser;

#[derive(Debug, Clone, Default)]
pub struct DocumentationMetrics {
    pub coverage_percent: f64,
    pub documented_items: usize,
    pub total_items: usize,
    pub has_readme: bool,
    pub readme_size: usize,
    pub comment_density: f64,
}

pub fn analyze_documentation(source: &[u8], lang: Language) -> DocumentationMetrics {
    let mut parser = Parser::new();
    parser
        .set_language(&lang.tree_sitter_language())
        .expect("Failed to set language");

    let tree = match parser.parse(source, None) {
        Some(t) => t,
        None => return DocumentationMetrics::default(),
    };

    let mut documented = 0;
    let mut total = 0;
    let mut comment_lines = 0;

    analyze_docs_node(
        tree.root_node(),
        source,
        lang,
        &mut documented,
        &mut total,
    );
    count_comments(tree.root_node(), source, &mut comment_lines);

    // Count non-empty code lines
    let source_str = std::str::from_utf8(source).unwrap_or("");
    let code_lines = source_str
        .lines()
        .filter(|l| !l.trim().is_empty())
        .count();

    let coverage = if total > 0 {
        (documented as f64 / total as f64) * 100.0
    } else {
        100.0 // No items to document = fully documented
    };

    let density = if code_lines > 0 {
        comment_lines as f64 / code_lines as f64
    } else {
        0.0
    };

    DocumentationMetrics {
        coverage_percent: coverage,
        documented_items: documented,
        total_items: total,
        has_readme: false, // Set at directory level
        readme_size: 0,
        comment_density: density,
    }
}

fn analyze_docs_node(
    node: tree_sitter::Node,
    source: &[u8],
    lang: Language,
    documented: &mut usize,
    total: &mut usize,
) {
    if is_documentable_item(&node, lang) {
        *total += 1;
        if has_doc_comment(&node, source, lang) {
            *documented += 1;
        }
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            analyze_docs_node(child, source, lang, documented, total);
        }
    }
}

fn is_documentable_item(node: &tree_sitter::Node, lang: Language) -> bool {
    let kind = node.kind();
    match lang {
        Language::TypeScript | Language::JavaScript => {
            matches!(
                kind,
                "function_declaration"
                    | "class_declaration"
                    | "method_definition"
                    | "export_statement"
            )
        }
        Language::Python => {
            matches!(kind, "function_definition" | "class_definition")
        }
        Language::Rust => {
            matches!(
                kind,
                "function_item"
                    | "struct_item"
                    | "enum_item"
                    | "impl_item"
                    | "trait_item"
                    | "mod_item"
            )
        }
        Language::Go => {
            matches!(
                kind,
                "function_declaration" | "method_declaration" | "type_declaration"
            )
        }
    }
}

fn has_doc_comment(node: &tree_sitter::Node, source: &[u8], lang: Language) -> bool {
    // Check previous sibling for doc comment
    if let Some(prev) = node.prev_sibling() {
        let kind = prev.kind();
        let text = prev.utf8_text(source).unwrap_or("");

        match lang {
            Language::TypeScript | Language::JavaScript => {
                kind == "comment" && text.starts_with("/**")
            }
            Language::Python => {
                // Python docstrings are inside the function body
                // Check first child for expression_statement containing string
                has_python_docstring(node, source)
            }
            Language::Rust => kind == "line_comment" && text.starts_with("///"),
            Language::Go => kind == "comment" && text.starts_with("//"),
        }
    } else if lang == Language::Python {
        // No previous sibling, but still check for docstring inside
        has_python_docstring(node, source)
    } else {
        false
    }
}

fn has_python_docstring(node: &tree_sitter::Node, source: &[u8]) -> bool {
    // Find the block child
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "block" {
                // Check first statement in block
                if let Some(first_stmt) = child.child(0) {
                    if first_stmt.kind() == "expression_statement" {
                        if let Some(expr) = first_stmt.child(0) {
                            if expr.kind() == "string" {
                                let text = expr.utf8_text(source).unwrap_or("");
                                // Check for triple-quoted string
                                return text.starts_with("\"\"\"")
                                    || text.starts_with("'''");
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

fn count_comments(node: tree_sitter::Node, source: &[u8], count: &mut usize) {
    let kind = node.kind();
    if kind == "comment" || kind == "line_comment" || kind == "block_comment" {
        let text = node.utf8_text(source).unwrap_or("");
        *count += text.lines().count();
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            count_comments(child, source, count);
        }
    }
}

pub fn check_readme(dir: &std::path::Path) -> (bool, usize) {
    let readme_names = ["README.md", "README.txt", "README", "readme.md", "Readme.md"];
    for name in readme_names {
        let path = dir.join(name);
        if path.exists() {
            if let Ok(metadata) = std::fs::metadata(&path) {
                return (true, metadata.len() as usize);
            }
        }
    }
    (false, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_file() {
        let metrics = analyze_documentation(b"", Language::TypeScript);
        assert_eq!(metrics.total_items, 0);
        assert_eq!(metrics.coverage_percent, 100.0); // No items = 100%
    }

    #[test]
    fn test_undocumented_function_ts() {
        let code = b"function hello() { return 'hello'; }";
        let metrics = analyze_documentation(code, Language::TypeScript);
        assert_eq!(metrics.total_items, 1);
        assert_eq!(metrics.documented_items, 0);
        assert_eq!(metrics.coverage_percent, 0.0);
    }

    #[test]
    fn test_documented_function_ts() {
        let code = b"/** Greets the user */\nfunction hello() { return 'hello'; }";
        let metrics = analyze_documentation(code, Language::TypeScript);
        assert_eq!(metrics.total_items, 1);
        assert_eq!(metrics.documented_items, 1);
        assert_eq!(metrics.coverage_percent, 100.0);
    }

    #[test]
    fn test_partial_documentation_ts() {
        let code = b"/** Documented */\nfunction a() {}\nfunction b() {}";
        let metrics = analyze_documentation(code, Language::TypeScript);
        assert_eq!(metrics.total_items, 2);
        assert_eq!(metrics.documented_items, 1);
        assert_eq!(metrics.coverage_percent, 50.0);
    }

    #[test]
    fn test_python_docstring() {
        let code = b"def hello():\n    \"\"\"Says hello\"\"\"\n    return 'hello'";
        let metrics = analyze_documentation(code, Language::Python);
        assert_eq!(metrics.total_items, 1);
        assert_eq!(metrics.documented_items, 1);
    }

    #[test]
    fn test_undocumented_python() {
        let code = b"def hello():\n    return 'hello'";
        let metrics = analyze_documentation(code, Language::Python);
        assert_eq!(metrics.total_items, 1);
        assert_eq!(metrics.documented_items, 0);
    }

    #[test]
    fn test_rust_doc_comment() {
        let code = b"/// Says hello\nfn hello() -> String { String::new() }";
        let metrics = analyze_documentation(code, Language::Rust);
        assert_eq!(metrics.total_items, 1);
        assert_eq!(metrics.documented_items, 1);
    }

    #[test]
    fn test_go_doc_comment() {
        let code = b"// Hello says hello\nfunc Hello() string { return \"hello\" }";
        let metrics = analyze_documentation(code, Language::Go);
        assert_eq!(metrics.total_items, 1);
        assert_eq!(metrics.documented_items, 1);
    }

    #[test]
    fn test_comment_density() {
        let code = b"// comment line\nfunction a() {}\n// another comment\nfunction b() {}";
        let metrics = analyze_documentation(code, Language::TypeScript);
        // 4 non-empty lines, 2 comment lines
        assert!(metrics.comment_density > 0.0);
    }

    #[test]
    fn test_class_documentation_ts() {
        let code = b"/** My class */\nclass Foo {}";
        let metrics = analyze_documentation(code, Language::TypeScript);
        assert_eq!(metrics.total_items, 1);
        assert_eq!(metrics.documented_items, 1);
    }

    #[test]
    fn test_struct_documentation_rust() {
        let code = b"/// My struct\nstruct Foo {}";
        let metrics = analyze_documentation(code, Language::Rust);
        assert_eq!(metrics.total_items, 1);
        assert_eq!(metrics.documented_items, 1);
    }

    #[test]
    fn test_check_readme_exists() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::write(dir.path().join("README.md"), "# Hello").unwrap();
        let (exists, size) = check_readme(dir.path());
        assert!(exists);
        assert!(size > 0);
    }

    #[test]
    fn test_check_readme_missing() {
        let dir = tempfile::TempDir::new().unwrap();
        let (exists, size) = check_readme(dir.path());
        assert!(!exists);
        assert_eq!(size, 0);
    }
}
