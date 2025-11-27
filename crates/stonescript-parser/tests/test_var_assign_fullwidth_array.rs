//! Test variable assignment with fullwidth bracket array

#[test]
fn test_var_fullwidth_array_simple() {
    // Using fullwidth bracket after var = 
    let input = r#"var x = ［1, 2, 3］"#;
    
    let result = stonescript_parser::parse_source(input);
    assert!(result.is_ok(), "Failed to parse var with fullwidth array: {:?}", result.err());
}

#[test]
fn test_var_fullwidth_array_ascii_single() {
    let input = r#"var Rt = ［ascii
AAA
asciiend］"#;
    
    let result = stonescript_parser::parse_source(input);
    if let Err(ref e) = result {
        eprintln!("Error: {}", e);
    }
    assert!(result.is_ok(), "Failed to parse var with fullwidth bracket ASCII array");
}

#[test]
fn test_var_fullwidth_array_ascii_multiple() {
    let input = r#"var Rt = ［ascii
AAA
asciiend,ascii
BBB
asciiend］"#;
    
    let result = stonescript_parser::parse_source(input);
    if let Err(ref e) = result {
        eprintln!("Error: {}", e);
    }
    assert!(result.is_ok(), "Failed to parse var with fullwidth bracket ASCII array with multiple blocks");
}

#[test]
fn test_exact_killerrpg_pattern() {
    // Exact pattern from KillerRPG line 88-110
    let input = r#"var Rt = ［ascii
.__________________________.
|—————————REWARDS——————————|
|#####################     |
|###################       |
|#####################     |
|######################### |
|                          |
|                          |
|                          |
|##########################|
|__________________________|
'##########################'
asciiend,ascii
.__________________________.
|—————————REWARDS——————————|
|#####################     |
|###################       |
|#####################     |
|######################### |
|__________________________|
'##########################'
asciiend］"#;
    
    let result = stonescript_parser::parse_source(input);
    if let Err(ref e) = result {
        eprintln!("Error parsing exact KillerRPG pattern: {}", e);
    }
    assert!(result.is_ok(), "Failed to parse exact KillerRPG Rt pattern");
}

#[test]
fn test_ascii_with_spaces() {
    // ASCII block with trailing spaces
    let input = r#"var Rt = ［ascii
|#####################     |
|###################       |
asciiend］"#;
    
    let result = stonescript_parser::parse_source(input);
    if let Err(ref e) = result {
        eprintln!("Error: {}", e);
    }
    assert!(result.is_ok(), "Failed to parse ASCII with trailing spaces");
}

#[test]
fn test_killerrpg_first_block() {
    // Just the first block from KillerRPG
    let input = r#"var Rt = ［ascii
.__________________________.
|—————————REWARDS——————————|
|#####################     |
asciiend］"#;
    
    let result = stonescript_parser::parse_source(input);
    if let Err(ref e) = result {
        eprintln!("Error: {}", e);
    }
    assert!(result.is_ok(), "Failed to parse first block");
}

#[test]
fn test_killerrpg_full_first_block() {
    // Full first block from KillerRPG
    let input = r#"var Rt = ［ascii
.__________________________.
|—————————REWARDS——————————|
|#####################     |
|###################       |
|#####################     |
|######################### |
|                          |
|                          |
|                          |
|##########################|
|__________________________|
'##########################'
asciiend］"#;
    
    let result = stonescript_parser::parse_source(input);
    if let Err(ref e) = result {
        eprintln!("Error parsing full first block: {}", e);
    }
    assert!(result.is_ok(), "Failed to parse full first block");
}

#[test]
fn test_killerrpg_two_blocks_minimal() {
    // Minimal two blocks
    let input = r#"var Rt = ［ascii
AAA
BBB
CCC
DDD
EEE
FFF
GGG
HHH
III
JJJ
KKK
asciiend,ascii
XXX
YYY
ZZZ
asciiend］"#;
    
    let result = stonescript_parser::parse_source(input);
    if let Err(ref e) = result {
        eprintln!("Error: {}", e);
    }
    assert!(result.is_ok(), "Failed to parse two blocks minimal");
}

#[test]
fn test_killerrpg_gradual_expansion() {
    // Start with first block + minimal second
    let input = r#"var Rt = ［ascii
.__________________________.
|—————————REWARDS——————————|
|#####################     |
|###################       |
|#####################     |
|######################### |
|                          |
|                          |
|                          |
|##########################|
|__________________________|
'##########################'
asciiend,ascii
.__________________________.
asciiend］"#;
    
    let result = stonescript_parser::parse_source(input);
    if let Err(ref e) = result {
        eprintln!("Error: {}", e);
    }
    assert!(result.is_ok(), "Failed: first full + second minimal");
}

#[test]
fn test_killerrpg_second_expanded() {
    let input = r#"var Rt = ［ascii
.__________________________.
|—————————REWARDS——————————|
|#####################     |
|###################       |
|#####################     |
|######################### |
|                          |
|                          |
|                          |
|##########################|
|__________________________|
'##########################'
asciiend,ascii
.__________________________.
|—————————REWARDS——————————|
|#####################     |
asciiend］"#;
    
    let result = stonescript_parser::parse_source(input);
    if let Err(ref e) = result {
        eprintln!("Error: {}", e);
    }
    assert!(result.is_ok(), "Failed: both blocks partial");
}

#[test]
fn test_smaller_first_block() {
    let input = r#"var Rt = ［ascii
.__________________________.
|—————————REWARDS——————————|
|#####################     |
|###################       |
asciiend,ascii
.__________________________.
asciiend］"#;
    
    let result = stonescript_parser::parse_source(input);
    if let Err(ref e) = result {
        eprintln!("Error: {}", e);
    }
    assert!(result.is_ok(), "Failed with smaller first block");
}

#[test]
fn test_medium_first_block() {
    let input = r#"var Rt = ［ascii
AAA
BBB
CCC
DDD
EEE
FFF
asciiend,ascii
XXX
asciiend］"#;
    
    let result = stonescript_parser::parse_source(input);
    if let Err(ref e) = result {
        eprintln!("Error: {}", e);
    }
    assert!(result.is_ok(), "Failed with medium first block");
}
