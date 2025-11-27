//! Test for Games/KillerRPG.txt parsing

use std::fs;
use std::path::PathBuf;

#[test]
fn test_killerrpg_file_parse() {
    // Find test_scripts directory
    let mut test_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_dir.pop(); // Go up from crates/stonescript-parser
    test_dir.pop(); // Go up from crates
    test_dir.push("test_scripts");
    test_dir.push("Games");
    test_dir.push("KillerRPG.txt");

    if !test_dir.exists() {
        println!("KillerRPG.txt not found at {:?}, skipping test", test_dir);
        return;
    }

    let content = fs::read_to_string(&test_dir).expect("Failed to read KillerRPG.txt");

    println!("Parsing KillerRPG.txt with {} bytes", content.len());

    // Test parsing
    let result = stonescript_parser::parse_source(&content);

    match result {
        Ok(program) => {
            println!("✓ Successfully parsed KillerRPG.txt");
            println!("  Statements: {}", program.statements.len());
        }
        Err(e) => {
            eprintln!("✗ Failed to parse KillerRPG.txt");
            eprintln!("Error: {}", e);

            // Print the area around the error
            if let Some(remaining) = e
                .to_string()
                .strip_prefix("Failed to parse completely. Remaining: ")
            {
                let truncated = if remaining.len() > 300 {
                    &remaining[..300]
                } else {
                    remaining
                };
                eprintln!("Remaining input (first 300 chars): {:?}", truncated);

                // Try to find the position in the original file
                if let Some(pos) = content.find(truncated) {
                    eprintln!("Error position: byte offset {}", pos);
                    let line_num = content[..pos].lines().count();
                    eprintln!("Approximate line number: {}", line_num);
                }
            }

            panic!("Failed to parse KillerRPG.txt: {}", e);
        }
    }
}
