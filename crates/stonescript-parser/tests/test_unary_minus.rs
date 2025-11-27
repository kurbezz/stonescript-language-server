//! Test unary minus operator in various contexts
//!
//! StoneScript should support unary minus in expressions, including:
//! - At the start of expressions: x = -5
//! - After operators: x = 10 + -5
//! - With whitespace: x = - 5 or x =  - 5
//! - In function calls: func(-x)
//! - In array literals: [1, -2, 3]

use stonescript_parser::parse_source;

#[test]
fn test_unary_minus_simple() {
    let input = r#"
x = -5
y = -3.14
"#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Failed to parse simple unary minus: {:?}",
        result.err()
    );
}

#[test]
fn test_unary_minus_with_space() {
    let input = r#"
x = - 5
y = -10
"#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Failed to parse unary minus with space: {:?}",
        result.err()
    );
}

#[test]
fn test_unary_minus_with_multiple_spaces() {
    let input = r#"
LStateListText.y =  - MaxListLength/2
RStateListText.y = -MaxListLength/2
"#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Failed to parse unary minus with multiple spaces: {:?}",
        result.err()
    );
}

#[test]
fn test_unary_minus_in_expression() {
    let input = r#"
x = 10 + -5
y = 20 * -2
z = -x + y
"#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Failed to parse unary minus in expression: {:?}",
        result.err()
    );
}

#[test]
fn test_unary_minus_in_function_call() {
    let input = r#"
result = math.Max(-5, 10)
value = func(-x, -y, z)
"#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Failed to parse unary minus in function call: {:?}",
        result.err()
    );
}

#[test]
fn test_unary_minus_in_array() {
    let input = r#"
arr = [1, -2, 3, -4]
matrix = [[1, -1], [-2, 2]]
"#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Failed to parse unary minus in array: {:?}",
        result.err()
    );
}

#[test]
fn test_unary_minus_in_property_access() {
    let input = r#"
obj.x = -10
obj.y = - obj.height / 2
"#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Failed to parse unary minus with property access: {:?}",
        result.err()
    );
}

#[test]
fn test_unary_minus_in_condition() {
    let input = r#"
?x = -5
  >`negative five
?y < -10
  >`very negative
"#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Failed to parse unary minus in condition: {:?}",
        result.err()
    );
}

#[test]
fn test_unary_minus_parenthesized() {
    let input = r#"
x = (-5)
y = (10 + (-3))
z = -(x + y)
"#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Failed to parse unary minus with parentheses: {:?}",
        result.err()
    );
}

#[test]
fn test_double_negation() {
    let input = r#"
x = - -5
y = --10
"#;
    let result = parse_source(input);
    // This might be an edge case - double negation
    // We just check it doesn't crash the parser
    let _ = result;
}

#[test]
fn test_unary_minus_with_indexing() {
    let input = r#"
x = -arr[0]
y = -list[i + 1]
"#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Failed to parse unary minus with indexing: {:?}",
        result.err()
    );
}

#[test]
fn test_unary_minus_with_method_call() {
    let input = r#"
x = -obj.GetValue()
y = -list.Count()
"#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Failed to parse unary minus with method call: {:?}",
        result.err()
    );
}
