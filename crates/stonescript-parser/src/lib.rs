//! StoneScript Parser
//!
//! Wrapper around tree-sitter-stonescript parser

use tree_sitter::{Language, Parser, Tree};

extern "C" {
    fn tree_sitter_stonescript() -> Language;
}

/// Get the tree-sitter Language for StoneScript
pub fn language() -> Language {
    unsafe { tree_sitter_stonescript() }
}

/// Create a new parser configured for StoneScript
pub fn parser() -> Parser {
    let mut parser = Parser::new();
    parser
        .set_language(language())
        .expect("Failed to load StoneScript language");
    parser
}

/// Parse StoneScript source code
pub fn parse(source: &str) -> Option<Tree> {
    let mut parser = parser();
    // Strip UTF-8 BOM if present
    let source = source.strip_prefix('\u{FEFF}').unwrap_or(source);
    parser.parse(source, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let parser = parser();
        assert_eq!(parser.language().unwrap(), language());
    }

    #[test]
    fn test_parse_variable() {
        let tree = parse("var x = 10").expect("Failed to parse");
        let root = tree.root_node();
        assert_eq!(root.kind(), "source_file");
        assert!(root.child_count() > 0);
    }

    #[test]
    fn test_parse_conditional() {
        let source = "?hp < 10\n  activate potion";
        let tree = parse(source).expect("Failed to parse");
        let root = tree.root_node();

        // Find conditional node
        let conditional = root.child(0).expect("No child node");
        assert_eq!(conditional.kind(), "conditional");
    }

    #[test]
    fn test_parse_function() {
        let source = "func test(a, b)\n  return a + b";
        let tree = parse(source).expect("Failed to parse");
        let root = tree.root_node();

        let func_decl = root.child(0).expect("No function declaration");
        assert_eq!(func_decl.kind(), "function_declaration");
    }
}
