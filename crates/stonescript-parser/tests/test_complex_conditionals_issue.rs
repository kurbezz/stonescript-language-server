//! Test for complex nested conditionals issue
//!
//! Issue: Parser fails on deeply nested if-else-if chains with complex boolean expressions
//! Found in: UI/BorderClock.txt and many game scripts
//!
//! Example failing code:
//! ?hourColor="#0000FF"&minuteColor="#00FF00"&secondColor="#FF0000"
//!     ?hour=min&hour=sec
//!         hourColor="#rainFF"
//!     :?hour=min
//!         hourColor="#00FFFF"

use stonescript_parser::parse_source;

#[test]
fn test_deeply_nested_if_else() {
    let source = r#"? x < 10
    y = 1
:? x < 20
    y = 2
:? x < 30
    y = 3
:? x < 40
    y = 4
:
    y = 5"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse deeply nested if-else: {:?}",
        result.err()
    );
}

#[test]
fn test_complex_boolean_in_condition() {
    let source = r##"? hourColor="#0000FF" & minuteColor="#00FF00" & secondColor="#FF0000"
    x = 1
:
    x = 2"##;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse complex boolean condition: {:?}",
        result.err()
    );
}

#[test]
fn test_nested_if_in_elseif() {
    let source = r##"? hour = min & hour = sec
    hourColor = "#rainFF"
:? hour = min
    hourColor = "#00FFFF"
:? hour = sec
    hourColor = "#FF00FF"
:? min = sec
    minuteColor = "#FFFF00"
:
    x = 1"##;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse nested if in elseif: {:?}",
        result.err()
    );
}

#[test]
fn test_multiple_levels_nesting() {
    let source = r#"? x < 100
    ? y < 50
        ? z < 25
            result = 1
        :? z < 40
            result = 2
        :
            result = 3
    :? y < 75
        result = 4
    :
        result = 5
:
    result = 6"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse multiple levels of nesting: {:?}",
        result.err()
    );
}

#[test]
fn test_border_clock_pattern() {
    // Simplified version of the failing pattern from BorderClock.txt
    let source = r##"func Main()
    var hour = time.hour
    var min = time.minute
    var sec = time.second

    ? sec < screen.w
        Output(sec, 0, secondColor, S)
    :
        sec = sec - screen.w
        ? sec < screen.h
            Output(screen.w - 1, sec, secondColor, S)
        :
            sec = sec - screen.h
            ? sec < screen.w
                Output(screen.w - 1 - sec, screen.h - 1, secondColor, S)
            :
                sec = sec - screen.w
                ? sec < screen.h
                    Output(0, screen.h - 1 - sec, secondColor, S)
                :
                    WTF = 1"##;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse BorderClock pattern: {:?}",
        result.err()
    );
}

#[test]
fn test_chained_elseif_with_complex_conditions() {
    let source = r#"? a = 1 & b = 2
    x = 1
:? a = 1 & c = 3
    x = 2
:? b = 2 & c = 3
    x = 3
:? a = 1 | b = 2 | c = 3
    x = 4
:
    x = 5"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse chained elseif with complex conditions: {:?}",
        result.err()
    );
}

#[test]
fn test_nested_if_with_function_calls() {
    let source = r#"? GetValue(x) = 10
    ? Calculate(y) > 5
        ? Process(z) = true
            result = 1
        :
            result = 2
    :
        result = 3
:
    result = 4"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse nested if with function calls: {:?}",
        result.err()
    );
}

#[test]
fn test_multiple_conditions_per_level() {
    let source = r#"? x = 1
    ? y = 2
        z = 3
    ? a = 4
        b = 5
:? x = 6
    ? c = 7
        d = 8
:
    e = 9"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse multiple conditions per level: {:?}",
        result.err()
    );
}

#[test]
fn test_deeply_nested_with_statements() {
    let source = r#"? level = 1
    x = 1
    ? level = 2
        y = 2
        z = 3
        ? level = 3
            a = 4
            b = 5
            ? level = 4
                c = 6
            :
                c = 7
        :
            a = 8
    :
        y = 9
:
    x = 10"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse deeply nested with statements: {:?}",
        result.err()
    );
}

#[test]
fn test_complex_nested_with_all_operators() {
    let source = r#"? (x > 0) & (y < 100)
    ? (a = b) | (c ! d)
        ? (e >= f) & (g <= h)
            result = 1
        :? (i > j) | (k < l)
            result = 2
        :
            result = 3
    :? (m = n) & (o ! p)
        result = 4
    :
        result = 5
:? (q >= r) | (s <= t)
    result = 6
:
    result = 7"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse complex nested with all operators: {:?}",
        result.err()
    );
}

#[test]
fn test_nested_conditions_with_property_access() {
    let source = r##"? foe.hp > 0
    ? player.armor >= foe.damage
        ? foe.name = "Boss"
            strategy = "defensive"
        :? foe.hp < 100
            strategy = "aggressive"
        :
            strategy = "normal"
    :
        strategy = "heal"
:
    strategy = "idle""##;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse nested conditions with property access: {:?}",
        result.err()
    );
}

#[test]
fn test_elseif_chain_ten_levels() {
    let source = r#"? x = 1
    y = 1
:? x = 2
    y = 2
:? x = 3
    y = 3
:? x = 4
    y = 4
:? x = 5
    y = 5
:? x = 6
    y = 6
:? x = 7
    y = 7
:? x = 8
    y = 8
:? x = 9
    y = 9
:? x = 10
    y = 10
:
    y = 0"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse elseif chain with 10 levels: {:?}",
        result.err()
    );
}

#[test]
fn test_mixed_nesting_and_chaining() {
    let source = r#"? a = 1
    ? b = 2
        ? c = 3
            x = 1
        :? c = 4
            x = 2
        :
            x = 3
    :? b = 5
        x = 4
    :
        x = 5
:? a = 6
    ? b = 7
        x = 6
    :
        x = 7
:
    x = 8"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse mixed nesting and chaining: {:?}",
        result.err()
    );
}

#[test]
fn test_conditions_with_negation() {
    let source = r#"? !flag1
    ? !flag2 & flag3
        ? !flag4 | flag5
            x = 1
        :
            x = 2
    :
        x = 3
:
    x = 4"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse conditions with negation: {:?}",
        result.err()
    );
}

#[test]
fn test_nested_with_return_statements() {
    let source = r#"func Test(x)
    ? x < 0
        return -1
    :? x = 0
        return 0
    :? x < 10
        ? x < 5
            return 1
        :
            return 2
    :? x < 100
        return 3
    :
        return 4"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse nested with return statements: {:?}",
        result.err()
    );
}
