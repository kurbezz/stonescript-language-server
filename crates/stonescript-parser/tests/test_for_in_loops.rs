//! Test for-in loops (iterating over collections)
//!
//! StoneScript supports two types of for loops:
//! 1. Range-based: for i = 0..10
//! 2. Collection-based (for-in): for element : collection

use stonescript_parser::{parse_source, Statement};

#[test]
fn test_simple_for_in_loop() {
    let input = r#"
for e : arr
  >`@e@
"#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Failed to parse simple for-in loop: {:?}",
        result.err()
    );

    let program = result.unwrap();
    // Filter out empty statements
    let non_empty: Vec<_> = program
        .statements
        .iter()
        .filter(|s| !matches!(s, Statement::Empty))
        .collect();
    assert_eq!(non_empty.len(), 1);

    match non_empty[0] {
        Statement::ForIn {
            variable,
            collection,
            body,
            ..
        } => {
            assert_eq!(variable, "e");
            assert_eq!(body.len(), 1);
        }
        _ => panic!("Expected ForIn statement, got {:?}", non_empty[0]),
    }
}

#[test]
fn test_for_in_with_nested_code() {
    let input = r#"
for item : items
  ?item > 0
    >`@item@
  :
    >`negative
"#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Failed to parse for-in with nested code: {:?}",
        result.err()
    );
}

#[test]
fn test_for_in_with_function_call() {
    let input = r#"
for e : arr
  ?Type(e) = "string"
    stringBuilder.Add(dquote + e + dquote)
  :?Type(e) = "array"
    stringBuilder.Add(Stringify(e))
  :
    stringBuilder.Add("" + e)
"#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Failed to parse for-in with function calls: {:?}",
        result.err()
    );
}

#[test]
fn test_for_in_with_method_calls() {
    let input = r#"
for i : List
  ?i[2] = 0
    TempText = TempText + i[4] + i[0] + ":" + i[1] + "[/color]\n"
  :
    TempText = TempText + i[4] + i[0] + ":" + i[1] + "(" + (i[1]+i[2]) + ")" + "[/color]\n"
"#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Failed to parse for-in with method calls: {:?}",
        result.err()
    );
}

#[test]
fn test_multiple_for_in_loops() {
    let input = r#"
for x : list1
  >`@x@

for y : list2
  >`@y@
"#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Failed to parse multiple for-in loops: {:?}",
        result.err()
    );

    let program = result.unwrap();
    // Filter out empty statements
    let non_empty: Vec<_> = program
        .statements
        .iter()
        .filter(|s| !matches!(s, Statement::Empty))
        .collect();
    assert_eq!(non_empty.len(), 2);
}

#[test]
fn test_nested_for_in_loops() {
    let input = r#"
for outer : outerList
  for inner : innerList
    >`@outer@ @inner@
"#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Failed to parse nested for-in loops: {:?}",
        result.err()
    );
}

#[test]
fn test_for_in_with_array_indexing() {
    let input = r#"
for i = 0..arr.Count()-1
  ?Type(arr[i]) = "array"
    stringBuilder.Add(Printify_Main(arr[i], prefixStr + "[" + i + "]"))
"#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Failed to parse for with array indexing: {:?}",
        result.err()
    );
}

#[test]
fn test_for_in_empty_body() {
    let input = r#"
for e : arr
"#;
    // This might be an edge case - empty body should probably be an error
    let result = parse_source(input);
    // For now we just check it doesn't crash
    let _ = result;
}

#[test]
fn test_for_in_with_break_continue() {
    let input = r#"
for e : arr
  ?e = target
    break
  process(e)
"#;
    let result = parse_source(input);
    // Note: We may need to add support for break/continue statements
    let _ = result;
}
