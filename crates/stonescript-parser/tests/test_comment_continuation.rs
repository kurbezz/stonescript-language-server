// Test line continuation after comment

#[test]
fn test_continuation_after_comment() {
    let input = r#"
// comment
^x = 1
"#;
    let result = stonescript_parser::parse_source(input);
    // If appended, it becomes "// commentx = 1", so x=1 is not parsed.
    // If not appended, it becomes "// comment\nx = 1", so x=1 is parsed.
    
    // We expect x=1 to be parsed.
    // The result should contain an assignment statement.
    match result {
        Ok(program) => {
            println!("Program: {:?}", program);
            let has_assignment = program.statements.iter().any(|s| matches!(s, stonescript_parser::Statement::Assignment { .. }));
            assert!(has_assignment, "Should have parsed assignment");
        }
        Err(e) => panic!("Failed to parse: {}", e),
    }
}
