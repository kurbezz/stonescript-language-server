#[test]
fn test_comment_on_own_line() {
    let result = stonescript_parser::parse_source("// comment");
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[test]
fn test_var_then_comment_on_next_line() {
    let result = stonescript_parser::parse_source("var x = 5\n// comment");
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[test]
fn test_two_statements_on_separate_lines() {
    let result = stonescript_parser::parse_source("var x = 5\nvar y = 5");
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}
