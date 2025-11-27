#[test]
fn test_var_with_spaces_before_equals() {
    let result = stonescript_parser::parse_source("var x  = 5");
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[test]
fn test_var_with_many_spaces_before_equals() {
    let result = stonescript_parser::parse_source("var x       = 5");
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}
