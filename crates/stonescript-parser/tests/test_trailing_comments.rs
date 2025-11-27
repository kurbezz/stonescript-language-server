#[test]
fn test_simple_var_with_trailing_comment() {
    let result = stonescript_parser::parse_source("var x  // comment");
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[test]
fn test_var_assignment_with_trailing_comment() {
    let result = stonescript_parser::parse_source("var x = 5  // comment");  
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}
