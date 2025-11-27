//! Test for escaped quotes issue
//!
//! Issue: Parser fails on escaped quotes in string literals like "\""
//! Found in: UI/FreeChestTracker.txt and many other files
//!
//! Example failing code:
//! ? char1 = "\"" | char1 = "^"

use stonescript_parser::parse_source;

#[test]
fn test_escaped_quote_in_string() {
    let source = r#"? char1 = "\""
    x = 1"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse escaped quote: {:?}",
        result.err()
    );
}

#[test]
fn test_escaped_quote_in_condition() {
    let source = r#"? char1 = "\"" | char1 = "^"
    x = 1"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse escaped quote in condition: {:?}",
        result.err()
    );
}

#[test]
fn test_multiple_escaped_quotes() {
    let source = r#"var str = "\"\""
x = 1"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse multiple escaped quotes: {:?}",
        result.err()
    );
}

#[test]
fn test_escaped_quote_in_comparison() {
    let source = r#"? draw.GetSymbol(x, y) = "\""
    print("found quote")"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse escaped quote in comparison: {:?}",
        result.err()
    );
}

#[test]
fn test_escaped_backslash() {
    let source = r#"var str = "\\"
x = 1"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse escaped backslash: {:?}",
        result.err()
    );
}

#[test]
fn test_mixed_escapes() {
    let source = r#"var str = "test \" and \\ end"
x = 1"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse mixed escapes: {:?}",
        result.err()
    );
}

#[test]
fn test_escaped_quote_in_function_call() {
    let source = r#"print("\"")"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse escaped quote in function call: {:?}",
        result.err()
    );
}

#[test]
fn test_string_with_various_escapes() {
    let source = r#"var test = "line1\nline2\ttab\"quote"
x = 1"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse string with various escapes: {:?}",
        result.err()
    );
}

#[test]
fn test_free_chest_tracker_pattern() {
    // Simplified version of the actual failing pattern from FreeChestTracker.txt
    let source = r#"func Main()
    var char1 = draw.GetSymbol(x1, y1)
    var char2 = draw.GetSymbol(x2, y2)

    ? char1 = "\"" | char1 = "^"
        ? detectLock = MIDDLE
            chestLoc = LEFT
        :? chestLoc = LEFT
            chestLoc = MIDDLE
    :
        ? detectLock = LEFT
            detectLock = MIDDLE

    ? char2 = "\"" | char2 = "^"
        ? detectLock = MIDDLE
            chestLoc = MIDDLE
        :? chestLoc = MIDDLE
            chestLoc = RIGHT
    :
        ? detectLock = RIGHT
            detectLock = MIDDLE"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse FreeChestTracker pattern: {:?}",
        result.err()
    );
}

#[test]
fn test_escaped_quote_at_string_end() {
    let source = r#"var str = "test\""
x = 1"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse escaped quote at end: {:?}",
        result.err()
    );
}

#[test]
fn test_empty_string_after_escaped_quote() {
    let source = r#"? x = "\""
    ? y = ""
        z = 1"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse empty string after escaped quote: {:?}",
        result.err()
    );
}
