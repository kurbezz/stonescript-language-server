//! Test for Cosmetics/Mech.txt - array of ASCII blocks with special syntax

use std::fs;
use std::path::PathBuf;

#[test]
fn test_mech_file_parse() {
    // Find test_scripts directory
    let mut test_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_dir.pop(); // Go up from crates/stonescript-parser
    test_dir.pop(); // Go up from crates
    test_dir.push("test_scripts");
    test_dir.push("Cosmetics");
    test_dir.push("Mech.txt");

    if !test_dir.exists() {
        println!("Mech.txt not found at {:?}, skipping test", test_dir);
        return;
    }

    let content = fs::read_to_string(&test_dir).expect("Failed to read Mech.txt");

    println!("Parsing Mech.txt with {} bytes", content.len());

    // Test parsing
    let result = stonescript_parser::parse_source(&content);

    match result {
        Ok(program) => {
            println!("✓ Successfully parsed Mech.txt");
            println!("  Statements: {}", program.statements.len());

            // Check for specific variables that use array of ASCII blocks
            let has_mech_wlk = program.statements.iter().any(|stmt| {
                if let stonescript_parser::Statement::Assignment { target, .. } = stmt {
                    if let stonescript_parser::Expression::Identifier(name, _) = target {
                        name == "MechWlkR"
                    } else {
                        false
                    }
                } else {
                    false
                }
            });

            assert!(has_mech_wlk, "Should contain MechWlkR variable");
        }
        Err(e) => {
            eprintln!("✗ Failed to parse Mech.txt");
            eprintln!("Error: {}", e);

            // Print the area around the error
            if let Some(remaining) = e
                .to_string()
                .strip_prefix("Failed to parse completely. Remaining: ")
            {
                let truncated = if remaining.len() > 200 {
                    &remaining[..200]
                } else {
                    remaining
                };
                eprintln!("Remaining input (first 200 chars): {:?}", truncated);
            }

            panic!("Failed to parse Mech.txt: {}", e);
        }
    }
}

#[test]
fn test_ascii_array_syntax() {
    // Test the specific syntax: var name = [ascii...asciiend,ascii...asciiend]
    let code = r#"
var MechWalk = ［ascii
###
# #
###
asciiend,ascii
###
# #
###
asciiend］
"#;

    let result = stonescript_parser::parse_source(code);
    match result {
        Ok(program) => {
            println!("✓ ASCII array syntax parsed successfully");
            println!("Number of statements: {}", program.statements.len());
            for (i, stmt) in program.statements.iter().enumerate() {
                println!("Statement {}: {:?}", i, stmt);
            }
            // Don't assert for now, just check it parses
            assert!(
                program.statements.len() > 0,
                "Should have at least one statement"
            );
        }
        Err(e) => {
            panic!("Failed to parse ASCII array: {}", e);
        }
    }
}

#[test]
fn test_ascii_array_with_comma() {
    // Test multiple ASCII blocks separated by commas
    let code = r#"
var frames = ［ascii
#1
asciiend,ascii
#2
asciiend,ascii
#3
asciiend］
"#;

    let result = stonescript_parser::parse_source(code);
    match result {
        Ok(_) => {
            println!("✓ Multiple ASCII blocks with commas parsed");
        }
        Err(e) => {
            panic!("Failed to parse multiple ASCII blocks: {}", e);
        }
    }
}
