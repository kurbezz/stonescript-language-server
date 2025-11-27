// Test case for fullwidth bracket array syntax issue
// Based on Games/KillerRPG.txt failure

use std::fs;
use std::path::PathBuf;

#[test]
fn test_fullwidth_bracket_arrays() {
    // Simple array with fullwidth brackets
    let input = r##"var LocArray = ［
Deadwood Canyon,
Temple,
Mushroom Florest
］
G = LocArray［0］
"##;
    
    let result = stonescript_parser::parse_source(input);
    if let Err(e) = &result {
        println!("Parse error: {:?}", e);
    }
    assert!(result.is_ok(), "Failed to parse fullwidth bracket array");
}

#[test]
fn test_multi_ascii_fullwidth_brackets() {
    // Multi-element ascii array with fullwidth brackets
    let input = r##"var Rt = ［ascii
Test content
asciiend,ascii
More test
asciiend］
"##;
    
    let result = stonescript_parser::parse_source(input);
    if let Err(e) = &result {
        println!("Parse error: {:?}", e);
    }
    assert!(result.is_ok(), "Failed to parse multi-ascii with fullwidth brackets");
}

#[test]
fn test_nested_fullwidth_arrays() {
    // Nested array access with fullwidth brackets
    let input = r##"var P =［
^［＂3rd Class Soldier＂,10,#00FFFF］,
^［＂2nd Class Soldier＂,20,#00FFFF］
^］
"##;
    
    let result = stonescript_parser::parse_source(input);
    if let Err(e) = &result {
        println!("Parse error: {:?}", e);
    }
    assert!(result.is_ok(), "Failed to parse nested fullwidth bracket arrays");
}

#[test]
fn test_killer_rpg_file() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // Go up from crates/stonescript-parser
    path.pop(); // Go up from crates
    path.push("test_scripts/Games/KillerRPG.txt");
    
    if !path.exists() {
        println!("Skipping test - file not found: {:?}", path);
        return;
    }
    
    let input = fs::read_to_string(&path).expect("Failed to read file");
    
    let result = stonescript_parser::parse_source(&input);
    if let Err(e) = &result {
        eprintln!("Parse error: {}", e);
        println!("Parse error: {}", e);
        eprintln!("Error occurred in file with {} bytes", input.len());
        
        // Try to find where it failed
        if let Some(remaining_text) = e.to_string().split("Remaining: \"").nth(1) {
            let first_100 = &remaining_text.chars().take(200).collect::<String>();
            eprintln!("First 200 chars of remaining: {}", first_100);
        }
    }
    assert!(result.is_ok(), "Failed to parse Games/KillerRPG.txt");
}
