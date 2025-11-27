#[test]
fn test_var_no_assignment_no_comment() {
    let result = stonescript_parser::parse_source("var x");
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[test]
fn test_var_with_assignment_no_comment() {
    let result = stonescript_parser::parse_source("var x = 5");
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[test]
fn test_var_no_assignment_with_comment() {
    let result = stonescript_parser::parse_source("var x // comment");
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[test]
fn test_var_with_assignment_and_comment() {
    let result = stonescript_parser::parse_source("var x = 5 // comment");
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[test]
fn test_var_with_assignment_and_comment_with_extra_spaces() {
    let result = stonescript_parser::parse_source("var x = 5  // comment");
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}
