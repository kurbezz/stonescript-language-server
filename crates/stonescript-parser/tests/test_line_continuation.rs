#[test]
fn test_line_continuation_in_function_call() {
    let input = r#"func SetNewTarget(SS, instance, input, inputSize, len)
  var result = string.IndexOf(input,
  ^                       " ", next)
  ?result = -1
    return 0
  return next
"#;
    
    let result = stonescript_parser::parse_source(input);
    assert!(result.is_ok(), "Failed to parse line continuation: {:?}", result.err());
}

#[test]
fn test_line_continuation_in_assignment() {
    let input = r#"var output = string.Sub(input,
^    indexST[instance], len)
"#;
    
    let result = stonescript_parser::parse_source(input);
    assert!(result.is_ok(), "Failed to parse line continuation in assignment: {:?}", result.err());
}

#[test]
fn test_multiple_line_continuations() {
    let input = r#"targetST[instance] =
^SetNewTarget(smartScroll,
^           instance, input,
^             inputSize, len)
"#;
    
    let result = stonescript_parser::parse_source(input);
    assert!(result.is_ok(), "Failed to parse multiple line continuations: {:?}", result.err());
}
