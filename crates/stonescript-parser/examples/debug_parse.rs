use stonescript_parser::parse_source;

fn main() {
    let test_file = std::env::args().nth(1).unwrap_or_else(|| {
        "test_scripts/Fishing.txt".to_string()
    });
    
    let content = std::fs::read_to_string(&test_file).expect("Failed to read file");
    
    println!("Parsing: {}", test_file);
    println!("Content length: {} bytes", content.len());
    println!("First 200 chars: {:?}\n", &content.chars().take(200).collect::<String>());
    
    match parse_source(&content) {
        Ok(program) => {
            println!("✓ Successfully parsed!");
            println!("Statements: {}", program.statements.len());
        }
        Err(e) => {
            eprintln!("✗ Parse error: {}", e);
            std::process::exit(1);
        }
    }
}
