//! Test for Unicode strings issue
//!
//! Issue: Parser fails on Unicode characters in string assignments
//! Found in: UI/OkamiroyUtils.txt and many cosmetic scripts
//!
//! Example failing code:
//! var MAX_SOLID_BAR = ██████████████████████████████████████████████████
//! var MAX_EMPTY_BAR = __________________________________________________

use stonescript_parser::parse_source;

#[test]
fn test_unicode_block_characters() {
    let source = r#"var MAX_SOLID_BAR = ██████████████████████████████████████████████████
x = 1"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse Unicode block characters: {:?}",
        result.err()
    );
}

#[test]
fn test_unicode_block_with_underscore() {
    let source = r#"var MAX_SOLID_BAR = ██████████████████████████████████████████████████
var MAX_EMPTY_BAR = __________________________________________________
x = 1"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse Unicode block with underscore: {:?}",
        result.err()
    );
}

#[test]
fn test_mixed_ascii_unicode() {
    let source = r#"var bar = "HP: ████____"
x = 1"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse mixed ASCII and Unicode: {:?}",
        result.err()
    );
}

#[test]
fn test_unicode_in_assignment() {
    let source = r#"var symbol = "█"
var empty = "_"
x = symbol"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse Unicode in assignment: {:?}",
        result.err()
    );
}

#[test]
fn test_unicode_emoji() {
    let source = r#"var face = "😀"
var heart = "❤️"
x = 1"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse Unicode emoji: {:?}",
        result.err()
    );
}

#[test]
fn test_unicode_arrows() {
    let source = r#"var up = "↑"
var down = "↓"
var left = "←"
var right = "→"
x = 1"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse Unicode arrows: {:?}",
        result.err()
    );
}

#[test]
fn test_unicode_box_drawing() {
    let source = r#"var corner = "┌"
var line = "─"
var vertical = "│"
x = 1"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse Unicode box drawing: {:?}",
        result.err()
    );
}

#[test]
fn test_unicode_in_function_param() {
    let source = r#"func DrawBar(solid)
    var bar = "████"
    print(bar)

DrawBar("██")"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse Unicode in function param: {:?}",
        result.err()
    );
}

#[test]
fn test_okamiroy_utils_pattern() {
    // Simplified version of the actual failing pattern from OkamiroyUtils.txt
    let source = r#"var MAX_SOLID_BAR = ██████████████████████████████████████████████████
var MAX_EMPTY_BAR = __________________________________________________

func BuildBarStr(curVal, maxVal, barLen, showEmptyChar)
    var solidLen = 0
    solidLen = curVal * barLen
    solidLen = solidLen / maxVal

    var barStr = ""
    ? curVal >= maxVal
        barStr = string.Sub(MAX_SOLID_BAR, 0, barLen)
    ? curVal < maxVal
        var solidPart = ""
        solidPart = string.Sub(MAX_SOLID_BAR, 0, solidLen)
        var emptyPart = ""
        emptyPart = string.Sub(MAX_EMPTY_BAR, 0, solidLen)
        barStr = solidPart + emptyPart

    return barStr"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse OkamiroyUtils pattern: {:?}",
        result.err()
    );
}

#[test]
fn test_unicode_string_concatenation() {
    let source = r#"var part1 = "████"
var part2 = "____"
var combined = part1 + part2
x = 1"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse Unicode string concatenation: {:?}",
        result.err()
    );
}

#[test]
fn test_unicode_in_output_statement() {
    let source = r#">5,5,#FF0000,████
x = 1"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse Unicode in output statement: {:?}",
        result.err()
    );
}

#[test]
fn test_unicode_in_draw_call() {
    let source = r#"draw.Text(10, 10, "█████", #FFFFFF)
x = 1"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse Unicode in draw call: {:?}",
        result.err()
    );
}

#[test]
fn test_wide_unicode_characters() {
    let source = r#"var chinese = "测试"
var japanese = "テスト"
var korean = "테스트"
x = 1"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse wide Unicode characters: {:?}",
        result.err()
    );
}

#[test]
fn test_unicode_in_ascii_block() {
    let source = r#"var art = ascii
████████
██    ██
████████
asciiend
x = 1"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse Unicode in ASCII block: {:?}",
        result.err()
    );
}

#[test]
fn test_multiple_unicode_types() {
    let source = r#"var blocks = "█▓▒░"
var shapes = "●◐○"
var stars = "★☆✦"
x = 1"#;

    let result = parse_source(source);
    assert!(
        result.is_ok(),
        "Failed to parse multiple Unicode types: {:?}",
        result.err()
    );
}
