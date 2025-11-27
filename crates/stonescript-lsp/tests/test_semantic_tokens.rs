//! Test semantic tokens generation

use stonescript_lsp::providers::semantic_tokens::SemanticTokensProvider;
use stonescript_parser::parse;

#[test]
fn test_semantic_tokens_generation() {
    let source = r#"// Test file
var x = 10
var name = "Test"

func greet(user)
  var message = "Hello"
  return true

?loc.begin
  activate
  equip sword

for i = 1..10
  x += i
"#;

    let ast = parse(source).expect("Failed to parse test source");
    let provider = SemanticTokensProvider::new();
    let tokens = provider.provide_semantic_tokens(&ast, source);

    println!("\nGenerated {} semantic tokens", tokens.data.len());

    // Print token details for debugging
    let mut current_line = 0;
    let mut current_col = 0;

    for (i, token) in tokens.data.iter().enumerate() {
        current_line += token.delta_line;
        if token.delta_line > 0 {
            current_col = token.delta_start;
        } else {
            current_col += token.delta_start;
        }

        let token_type_name = match token.token_type {
            0 => "VARIABLE",
            1 => "FUNCTION",
            2 => "KEYWORD",
            3 => "OPERATOR",
            4 => "NUMBER",
            5 => "STRING",
            6 => "COMMENT",
            7 => "PARAMETER",
            8 => "PROPERTY",
            _ => "UNKNOWN",
        };

        println!(
            "Token {:3}: line={:2} col={:3} len={:2} type={} ({})",
            i, current_line, current_col, token.length, token.token_type, token_type_name
        );
    }

    // We should have generated some tokens
    assert!(
        !tokens.data.is_empty(),
        "No semantic tokens were generated! Check the implementation."
    );
}

#[test]
fn test_token_types() {
    let source = r#"var x = 10
var name = "hello"
func test()
  return true
"#;

    let ast = parse(source).expect("Failed to parse");
    let provider = SemanticTokensProvider::new();
    let tokens = provider.provide_semantic_tokens(&ast, source);

    // Should have tokens for: var (keyword), x (variable), 10 (number),
    // var (keyword), name (variable), "hello" (string),
    // func (keyword), test (function), return (keyword), true (keyword)

    let token_types: Vec<u32> = tokens.data.iter().map(|t| t.token_type).collect();

    // Check we have different token types
    assert!(token_types.contains(&0), "Should have VARIABLE tokens");
    assert!(token_types.contains(&2), "Should have KEYWORD tokens");
    assert!(token_types.contains(&4), "Should have NUMBER tokens");
    assert!(token_types.contains(&5), "Should have STRING tokens");
}
