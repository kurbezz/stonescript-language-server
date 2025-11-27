//! Test for string assignment parsing issues

#[test]
fn test_simple_string_assignment() {
    let code = r#"x = "test""#;
    let result = stonescript_parser::parse_source(code);
    assert!(
        result.is_ok(),
        "Failed to parse simple string assignment: {:?}",
        result.err()
    );
}

#[test]
fn test_string_with_escaped_quote() {
    let code = r#"x = "\"test\"""#;
    let result = stonescript_parser::parse_source(code);
    assert!(
        result.is_ok(),
        "Failed to parse string with escaped quote: {:?}",
        result.err()
    );
}

#[test]
fn test_string_comparison() {
    let code = r#"? x = "test"
  y = 1"#;
    let result = stonescript_parser::parse_source(code);
    assert!(
        result.is_ok(),
        "Failed to parse string comparison in condition: {:?}",
        result.err()
    );
}

#[test]
fn test_var_string_assignment() {
    let code = r#"var x = "test""#;
    let result = stonescript_parser::parse_source(code);
    assert!(
        result.is_ok(),
        "Failed to parse var with string: {:?}",
        result.err()
    );
}

#[test]
fn test_string_concatenation() {
    let code = r#"x = "a" + "b""#;
    let result = stonescript_parser::parse_source(code);
    assert!(
        result.is_ok(),
        "Failed to parse string concatenation: {:?}",
        result.err()
    );
}

#[test]
fn test_multiple_string_assignments() {
    let code = r#"x = "first"
y = "second"
z = "third""#;
    let result = stonescript_parser::parse_source(code);
    assert!(
        result.is_ok(),
        "Failed to parse multiple string assignments: {:?}",
        result.err()
    );
}

#[test]
fn test_string_with_special_chars() {
    let code = r#"x = "\"" | y = "^""#;
    let result = stonescript_parser::parse_source(code);
    assert!(
        result.is_ok(),
        "Failed to parse string with special chars: {:?}",
        result.err()
    );
}

#[test]
fn test_integer_assignment_works() {
    // This should work to confirm the issue is specific to strings
    let code = "x = 123";
    let result = stonescript_parser::parse_source(code);
    assert!(
        result.is_ok(),
        "Integer assignment should work: {:?}",
        result.err()
    );
}

#[test]
fn test_var_integer_assignment() {
    let code = "var x = 123";
    let result = stonescript_parser::parse_source(code);
    assert!(
        result.is_ok(),
        "Var integer assignment failed: {:?}",
        result.err()
    );
}

#[test]
fn test_condition_with_else_block() {
    let code = r#"? x = 1
  y = 2
:
  z = 3"#;
    let result = stonescript_parser::parse_source(code);
    assert!(
        result.is_ok(),
        "Failed to parse condition with else block: {:?}",
        result.err()
    );
}

#[test]
fn test_condition_with_empty_else() {
    let code = r#"? x = 1
  y = 2
:"#;
    let result = stonescript_parser::parse_source(code);
    assert!(
        result.is_ok(),
        "Failed to parse condition with empty else: {:?}",
        result.err()
    );
}

#[test]
fn test_nested_conditions_with_string() {
    let code = r#"? char1 = "\"" | char1 = "^"
  ? detectLock = MIDDLE
    drawbg(x1, y1, detectCol)
:
  drawbg(x1, y1, baseCol)"#;
    let result = stonescript_parser::parse_source(code);
    assert!(
        result.is_ok(),
        "Failed to parse nested conditions with string: {:?}",
        result.err()
    );
}

#[test]
fn test_function_with_tabs() {
    let code = "func drawbg(xx, yy, col)\n\t? seamlessMode\n\t\treturn\n\tdraw.Bg(xx, yy, col)";
    let result = stonescript_parser::parse_source(code);
    assert!(
        result.is_ok(),
        "Failed to parse function with tab indentation: {:?}",
        result.err()
    );
}

#[test]
fn test_mixed_indentation() {
    let code = "? x = 1\n  y = 2\n\tz = 3";
    let result = stonescript_parser::parse_source(code);
    assert!(
        result.is_ok(),
        "Failed to parse mixed tab and space indentation: {:?}",
        result.err()
    );
}

#[test]
fn test_boolean_literals() {
    let code = r#"var x = true
var y = false
? x = true
  y = 1"#;
    let result = stonescript_parser::parse_source(code);
    assert!(
        result.is_ok(),
        "Failed to parse boolean literals: {:?}",
        result.err()
    );
}

#[test]
fn test_string_comparison_with_or() {
    let code = r#"? char1 = "\"" | char1 = "^"
  y = 1"#;
    let result = stonescript_parser::parse_source(code);
    assert!(
        result.is_ok(),
        "Failed to parse string comparison with OR: {:?}",
        result.err()
    );
}

#[test]
fn test_complex_nested_condition() {
    let code = r#"? loc = "test" & pos.x >= 52
  ? char1 = "\"" | char1 = "^"
    ? detectLock = MIDDLE
      x = 1
    :
      y = 2
  :
    z = 3"#;
    let result = stonescript_parser::parse_source(code);
    assert!(
        result.is_ok(),
        "Failed to parse complex nested condition: {:?}",
        result.err()
    );
}
