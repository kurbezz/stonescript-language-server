//! Test for arrays containing multiple ASCII blocks

#[test]
fn test_simple_ascii_array_single() {
    let input = r#"var frames = [ascii
AAA
BBB
asciiend]"#;
    
    let result = stonescript_parser::parse_source(input);
    assert!(result.is_ok(), "Failed to parse single ASCII in array: {:?}", result.err());
}

#[test]
fn test_simple_ascii_array_two_blocks() {
    let input = r#"var frames = [ascii
AAA
BBB
asciiend,ascii
CCC
DDD
asciiend]"#;
    
    let result = stonescript_parser::parse_source(input);
    assert!(result.is_ok(), "Failed to parse two ASCII blocks in array: {:?}", result.err());
}

#[test]
fn test_simple_ascii_array_three_blocks() {
    let input = r#"var frames = [ascii
AAA
asciiend,ascii
BBB
asciiend,ascii
CCC
asciiend]"#;
    
    let result = stonescript_parser::parse_source(input);
    assert!(result.is_ok(), "Failed to parse three ASCII blocks in array: {:?}", result.err());
}

#[test]
fn test_fullwidth_bracket_ascii_array() {
    let input = r#"var frames = ［ascii
AAA
asciiend,ascii
BBB
asciiend］"#;
    
    let result = stonescript_parser::parse_source(input);
    assert!(result.is_ok(), "Failed to parse fullwidth bracket ASCII array: {:?}", result.err());
}

#[test]
fn test_mech_walk_pattern() {
    // Simplified pattern from Cosmetics/Mech.txt
    let input = r#"var MechWlkR = ［ascii
#######.=≡=.
######/__ __\
asciiend,ascii
#######.=≡=.
######/__ __\
asciiend］"#;
    
    let result = stonescript_parser::parse_source(input);
    assert!(result.is_ok(), "Failed to parse Mech walk pattern: {:?}", result.err());
}
