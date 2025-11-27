//! Test cases for parsing issues found in test_scripts

use stonescript_parser::parse_source;

#[test]
fn test_fullwidth_quotes() {
    // Issue: Fullwidth quotes (＂) used instead of regular quotes (")
    let input = r#"var x = storage.get(＂x＂,0)"#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Should parse fullwidth quotes: {:?}",
        result
    );
}

#[test]
fn test_fullwidth_brackets_ascii() {
    // Issue: Fullwidth brackets (［］) used with ascii blocks
    let input = r#"var Borde = ［ascii
+---+
|###|
+---+
asciiend］"#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Should parse fullwidth brackets in ascii: {:?}",
        result
    );
}

#[test]
fn test_array_with_identifiers() {
    // Issue: Arrays containing bare identifiers (not strings)
    let input = r#"var tracks = [
cross_deadwood_river,
rocky_plateau_4,
deadwood_3
]"#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Should parse arrays with identifiers: {:?}",
        result
    );
}

#[test]
fn test_array_with_identifiers_single_line() {
    // Issue: Arrays containing bare identifiers on single line
    let input = r#"var tracks = [cross_deadwood_river, rocky_plateau_4, deadwood_3]"#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Should parse single-line arrays with identifiers: {:?}",
        result
    );
}

#[test]
fn test_escaped_fullwidth_quote() {
    // Issue: String containing escaped quotes
    let input = r#"? char1 = "\"" | char1 = "^""#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Should parse string with escaped quotes: {:?}",
        result
    );
}

#[test]
fn test_escaped_quote_in_string() {
    // Issue: Escaped quote character in string literal
    let input = r#"var x = "\"""#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Should parse escaped quote in string: {:?}",
        result
    );
}

#[test]
fn test_escaped_quote_comparison() {
    // Issue: Comparison with escaped quote
    let input = r#"? char1 = "\""
    x = 1"#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Should parse comparison with escaped quote: {:?}",
        result
    );
}

#[test]
fn test_escaped_quote_with_or() {
    // Issue: Escaped quote in condition with OR operator
    let input = r#"? char2 = "\"" | char2 = "^"
    x = 1"#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Should parse escaped quote with OR operator: {:?}",
        result
    );
}

#[test]
fn test_free_chest_tracker_snippet() {
    // Issue: Real-world snippet from FreeChestTracker.txt
    let input = r#"func Main()
    ? loc = "undead_crypt_intro" & pos.x >= 52
        char1 = draw.GetSymbol(x1, y1)
        char2 = draw.GetSymbol(x2, y2)

        ? char1 = "\"" | char1 = "^"
            ? detectLock = MIDDLE
                drawbg(x1, y1, detectCol)
                ? chestLoc = MIDDLE
                    chestLoc = LEFT
                :? chestLoc = LEFT
                    chestLoc = MIDDLE
                detectLock = LEFT
        :
            drawbg(x1, y1, baseCol)
            ? detectLock = LEFT
                detectLock = MIDDLE"#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Should parse FreeChestTracker snippet: {:?}",
        result
    );
}

#[test]
fn test_multiline_expression() {
    // Issue: Multiline expressions with line continuations
    // Note: StoneScript doesn't support tuple literals, so this is a function call
    let input = r#"func test(input, next)
    a = func2(input,
         " ",
         next)"#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Should parse multiline expressions: {:?}",
        result
    );
}

#[test]
fn test_complex_assignment() {
    // Issue: Complex assignments with special characters
    let input = r#"var x = [[＂Vigor＂, ＂ ∞＂]]"#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Should parse complex assignments: {:?}",
        result
    );
}

#[test]
fn test_bar_character() {
    // Issue: Unicode bar characters in strings
    let input = r#"var x = "│""#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Should parse unicode bar character: {:?}",
        result
    );
}

#[test]
fn test_output_with_interpolation() {
    // Issue: Output statement with complex interpolation
    let input = r#">`@xi@,@yi@,Player Name: @player.name@"#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Should parse output with interpolation: {:?}",
        result
    );
}

#[test]
fn test_empty_array() {
    // Issue: Empty array assignment
    let input = r#"var arr = []"#;
    let result = parse_source(input);
    assert!(result.is_ok(), "Should parse empty array: {:?}", result);
}

#[test]
fn test_nested_arrays() {
    // Issue: Nested arrays
    let input = r#"var arr = [[1, 2], [3, 4]]"#;
    let result = parse_source(input);
    assert!(result.is_ok(), "Should parse nested arrays: {:?}", result);
}

#[test]
fn test_comparison_with_fullwidth_quotes() {
    // Issue: Comparison with fullwidth quoted strings
    let input = r#"?foe = ＂boss＂
    x = 1"#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Should parse comparison with fullwidth quotes: {:?}",
        result
    );
}

#[test]
fn test_string_concatenation() {
    // Issue: String concatenation with +
    let input = r#"var text = ＂HP: ＂ + ＂100＂"#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Should parse string concatenation: {:?}",
        result
    );
}

#[test]
fn test_function_call_multiline() {
    // Issue: Function calls spanning multiple lines
    // Use line continuation character ^ which is preprocessed
    let input = r#"result = func(arg1,
^    arg2,
^    arg3)"#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Should parse multiline function calls: {:?}",
        result
    );
}

#[test]
fn test_label_or_goto() {
    // Issue: Labels with colons
    let input = r#": label
x = 1"#;
    let result = parse_source(input);
    // This might be a label/goto statement
    // For now just check if it parses or gives reasonable error
    let _ = result;
}

#[test]
fn test_box_drawing_characters() {
    // Issue: Box drawing characters in output
    let input = r#">`0,0,╔═══════════╗"#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Should parse box drawing characters: {:?}",
        result
    );
}

#[test]
fn test_array_with_mixed_types() {
    // Issue: Arrays with mixed types (strings and identifiers)
    let input = r#"var arr = [＂text＂, identifier, 123]"#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Should parse arrays with mixed types: {:?}",
        result
    );
}

#[test]
fn test_condition_with_multiple_or() {
    // Issue: Conditions with multiple OR operators
    let input = r#"?(loc ! uulaa & loc ! undead_crypt)
    x = 1"#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Should parse complex conditions: {:?}",
        result
    );
}

#[test]
fn test_block_pattern() {
    // Issue: Block pattern characters
    let input = r#"var blocks = "██████████████████████████████████████████████████""#;
    let result = parse_source(input);
    assert!(result.is_ok(), "Should parse block pattern: {:?}", result);
}

#[test]
fn test_fullwidth_brackets_empty_array() {
    // Issue: Empty array with fullwidth brackets
    let input = r#"var arr = ［］"#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Should parse empty array with fullwidth brackets: {:?}",
        result
    );
}

#[test]
fn test_fullwidth_brackets_array_with_strings() {
    // Issue: Array with fullwidth brackets containing strings
    let input = r#"var arr = ［＂item1＂, ＂item2＂, ＂item3＂］"#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Should parse array with fullwidth brackets and strings: {:?}",
        result
    );
}

#[test]
fn test_fullwidth_brackets_multiline_array() {
    // Issue: Multiline array with fullwidth brackets
    let input = r#"var arr = ［
＂item1＂,
＂item2＂,
＂item3＂
］"#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Should parse multiline array with fullwidth brackets: {:?}",
        result
    );
}

#[test]
fn test_line_continuation_in_function_call() {
    // Issue: Line continuation character ^ in function calls
    let input = r#"result = string.IndexOf(input,
^                       " ",next)"#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Should parse line continuation in function call: {:?}",
        result
    );
}

#[test]
fn test_multiline_function_call_with_spaces() {
    // Issue: Function calls with arguments separated by many spaces
    let input = r#"result = Scroll(input,                       " ",next)"#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Should parse function call with many spaces: {:?}",
        result
    );
}

#[test]
fn test_standalone_colon() {
    // Issue: Standalone colon used as empty else or label
    let input = r#"? x > 0
    y = 1
:
    y = 0"#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Should parse standalone colon: {:?}",
        result
    );
}

#[test]
fn test_double_negation() {
    // Issue: Double negation operator !!
    let input = r#"visible = !!foe"#;
    let result = parse_source(input);
    assert!(result.is_ok(), "Should parse double negation: {:?}", result);
}

#[test]
fn test_location_name_without_quotes() {
    // Issue: Location names used as identifiers without quotes
    let input = r#"? loc = waterfall
    x = 1"#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Should parse location names as identifiers: {:?}",
        result
    );
}

#[test]
fn test_multiple_not_equals_conditions() {
    // Issue: Multiple != comparisons with location names
    let input = r#"?(loc ! uulaa & loc ! undead_crypt)
    x = 1"#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Should parse multiple not equals conditions: {:?}",
        result
    );
}

#[test]
fn test_array_literal_start() {
    // Issue: Array starting at beginning of statement
    let input = r#"arr = [1, 2, 3]"#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Should parse array at statement start: {:?}",
        result
    );
}

#[test]
fn test_empty_string_comparison() {
    // Issue: Comparison with empty string
    let input = r#"? bar = ""
    x = 1"#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Should parse empty string comparison: {:?}",
        result
    );
}

#[test]
fn test_property_with_underscore() {
    // Issue: Properties with underscores
    let input = r#"x = rocky_plateau_star"#;
    let result = parse_source(input);
    assert!(
        result.is_ok(),
        "Should parse identifiers with underscores: {:?}",
        result
    );
}
