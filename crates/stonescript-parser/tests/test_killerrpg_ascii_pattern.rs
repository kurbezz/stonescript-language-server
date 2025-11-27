//! Test specifically for KillerRPG ASCII pattern

#[test]
fn test_killerrpg_ascii_simple() {
    // Simplified pattern from KillerRPG
    let input = r#"var Rt = ［ascii
.__________________________.
|—————————REWARDS——————————|
asciiend,ascii
.__________________________.
|—————————REWARDS——————————|
asciiend］"#;

    let result = stonescript_parser::parse_source(input);
    if let Err(ref e) = result {
        eprintln!("Error: {:?}", e);
    }
    assert!(result.is_ok(), "Failed to parse KillerRPG ASCII pattern");
}

#[test]
fn test_em_dash_in_ascii() {
    // Test em dash character (—)
    let input = r#"var x = [ascii
|—————|
asciiend]"#;

    let result = stonescript_parser::parse_source(input);
    assert!(
        result.is_ok(),
        "Failed to parse ASCII with em dash: {:?}",
        result.err()
    );
}

#[test]
fn test_fullwidth_bracket_em_dash() {
    // Combination of fullwidth bracket and em dash
    let input = r#"var x = ［ascii
|—————|
asciiend,ascii
|—————|
asciiend］"#;

    let result = stonescript_parser::parse_source(input);
    assert!(
        result.is_ok(),
        "Failed to parse fullwidth bracket with em dash: {:?}",
        result.err()
    );
}
