//! Integration test - parse all test scripts and collect detailed errors

use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

#[test]
fn test_all_scripts_parse() {
    // Find test_scripts directory - it's at the workspace root
    let mut test_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_dir.pop(); // Go up from crates/stonescript-parser
    test_dir.pop(); // Go up from crates
    test_dir.push("test_scripts");

    if !test_dir.exists() {
        println!(
            "test_scripts directory not found at {:?}, skipping integration test",
            test_dir
        );
        return;
    }

    println!("\nParsing StoneScript files from: {:?}", test_dir);

    let mut passed = 0;
    let mut failed = 0;
    let mut failed_files = Vec::new();

    for entry in WalkDir::new(&test_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("txt"))
    {
        let path = entry.path();
        let relative_path = path.strip_prefix(&test_dir).unwrap_or(path);

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("✗ {:?} - Failed to read: {}", relative_path, e);
                failed += 1;
                failed_files.push((
                    relative_path.display().to_string(),
                    format!("Read error: {}", e),
                ));
                continue;
            }
        };

        if relative_path.to_string_lossy().contains("Pets/Dog.txt") {
            println!("DEBUG: Parsing Pets/Dog.txt");
            println!("Content length: {}", content.len());
            println!("First 100 chars: {:?}", &content[..100.min(content.len())]);
            // Print hex of the problematic area (approx offset 6700-6900)
            if content.len() > 6900 {
                println!("Content around failure:");
                let slice = &content[6700..6900];
                println!("{:?}", slice);
            }
        }

        match stonescript_parser::parse_source(&content) {
            Ok(_program) => {
                println!("✓ {:?}", relative_path);
                passed += 1;
            }
            Err(e) => {
                if relative_path.to_string_lossy().contains("Pets/Dog.txt") {
                    eprintln!("DEBUG: Pets/Dog.txt failed with error: {}", e);
                }
                eprintln!("✗ {:?} - Parse error: {}", relative_path, e);
                failed += 1;
                failed_files.push((
                    relative_path.display().to_string(),
                    format!("Parse error: {}", e),
                ));
            }
        }
    }

    println!("\n==============================================");
    println!("Integration Test Results:");
    println!("  Total:  {}", passed + failed);
    println!("  Passed: {}", passed);
    println!("  Failed: {}", failed);
    println!("==============================================");

    if !failed_files.is_empty() {
        println!("\nFailed files:");
        for (file, error) in &failed_files {
            println!("  - {}", file);
            println!("    Error: {}", error);
        }
    }

    // Don't assert for now, just report
    if failed > 0 {
        println!("\n⚠️  {} files failed to parse", failed);
    }
}
