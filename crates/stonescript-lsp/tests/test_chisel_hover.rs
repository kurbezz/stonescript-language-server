//! Test hover with real Chisel.txt file to reproduce issues

use stonescript_lsp::providers::hover::HoverProvider;
use stonescript_lsp::utils::ScopeAnalyzer;
use stonescript_parser::parse_source;
use tower_lsp::lsp_types::Position;

#[test]
fn test_hover_on_redo_function_in_chisel() {
    // Load the actual Chisel.txt file
    let source = std::fs::read_to_string("../../test_scripts/Chisel.txt")
        .expect("Failed to read Chisel.txt");
    
    let ast = parse_source(&source).expect("Failed to parse Chisel.txt");
    let mut scope = ScopeAnalyzer::new();
    scope.analyze_ast(&ast);
    
    let provider = HoverProvider::new();
    
    // Find the line with "func Redo()" - should be around line 661
    // Note: We need to search in the actual source, not in lines array
    let mut redo_line_in_source = None;
    let mut current_line = 0;
    for (byte_pos, _) in source.match_indices('\n') {
        let line_start = if current_line == 0 { 0 } else {
            source[..byte_pos].rfind('\n').map(|p| p + 1).unwrap_or(0)
        };
        let line_content = &source[line_start..byte_pos];
        if line_content.trim().starts_with("func Redo()") {
            redo_line_in_source = Some(current_line);
            println!("Found 'func Redo()' at line {} in source", current_line);
            break;
        }
        current_line += 1;
    }
    
    // Also find using lines() for comparison
    let lines: Vec<&str> = source.lines().collect();
    let mut redo_line_in_lines = None;
    for (i, line) in lines.iter().enumerate() {
        if line.trim().starts_with("func Redo()") {
            redo_line_in_lines = Some(i);
            println!("Found 'func Redo()' at line {} (via lines())", i);
            break;
        }
    }
    
    assert!(redo_line_in_source.is_some(), "Should find 'func Redo()' in Chisel.txt");
    let line_num = redo_line_in_source.unwrap();
    
    println!("Testing hover at line {} (0-indexed), character 7", line_num);
    if let Some(lines_idx) = redo_line_in_lines {
        println!("Line content (via lines()): {}", lines[lines_idx]);
    }
    
    // Hover on "Redo" in the function definition (column 5 is after "func ")
    let hover = provider.provide_hover(
        &ast,
        Position { line: line_num as u32, character: 7 },
        &source,
        &scope,
    );
    
    assert!(hover.is_some(), "Should have hover for Redo function");
    let hover = hover.unwrap();
    
    if let tower_lsp::lsp_types::HoverContents::Markup(content) = hover.contents {
        println!("Hover content: {}", content.value);
        assert!(content.value.contains("func Redo"), "Should show Redo function, got: {}", content.value);
        assert!(!content.value.contains("UpdateRedoButtonColour"), 
            "Should NOT show UpdateRedoButtonColour, got: {}", content.value);
        assert!(content.value.contains("User-defined function"), "Should indicate it's a function");
    } else {
        panic!("Expected Markup hover content");
    }
}

#[test]
fn test_hover_on_update_redo_button_colour_in_chisel() {
    // Load the actual Chisel.txt file
    let source = std::fs::read_to_string("../../test_scripts/Chisel.txt")
        .expect("Failed to read Chisel.txt");
    
    let ast = parse_source(&source).expect("Failed to parse Chisel.txt");
    let mut scope = ScopeAnalyzer::new();
    scope.analyze_ast(&ast);
    
    let provider = HoverProvider::new();
    
    // Find the line with "func UpdateRedoButtonColour()" - should be around line 669
    let lines: Vec<&str> = source.lines().collect();
    let mut func_line = None;
    for (i, line) in lines.iter().enumerate() {
        if line.trim().starts_with("func UpdateRedoButtonColour()") {
            func_line = Some(i);
            println!("Found 'func UpdateRedoButtonColour()' at line {}", i);
            break;
        }
    }
    
    assert!(func_line.is_some(), "Should find 'func UpdateRedoButtonColour()' in Chisel.txt");
    let line_num = func_line.unwrap();
    
    // Hover on "UpdateRedoButtonColour" in the function definition
    let hover = provider.provide_hover(
        &ast,
        Position { line: line_num as u32, character: 10 },
        &source,
        &scope,
    );
    
    assert!(hover.is_some(), "Should have hover for UpdateRedoButtonColour function");
    let hover = hover.unwrap();
    
    if let tower_lsp::lsp_types::HoverContents::Markup(content) = hover.contents {
        println!("Hover content: {}", content.value);
        assert!(content.value.contains("func UpdateRedoButtonColour"), 
            "Should show UpdateRedoButtonColour function, got: {}", content.value);
        assert!(!content.value.contains("func Redo()"), 
            "Should NOT show Redo function, got: {}", content.value);
        assert!(content.value.contains("User-defined function"), "Should indicate it's a function");
    } else {
        panic!("Expected Markup hover content");
    }
}

#[test]
fn test_hover_on_redo_call_in_chisel() {
    // Load the actual Chisel.txt file
    let source = std::fs::read_to_string("../../test_scripts/Chisel.txt")
        .expect("Failed to read Chisel.txt");
    
    let ast = parse_source(&source).expect("Failed to parse Chisel.txt");
    let mut scope = ScopeAnalyzer::new();
    scope.analyze_ast(&ast);
    
    let provider = HoverProvider::new();
    
    // Find a line where Redo is used as a callback in MakeButton
    // Line 292: toolRedo = MakeButton(9, 9, 8, 3, "Redo", null, Redo)
    let lines: Vec<&str> = source.lines().collect();
    let mut call_line = None;
    for (i, line) in lines.iter().enumerate() {
        if line.contains("MakeButton") && line.contains(", Redo)") {
            call_line = Some(i);
            println!("Found Redo callback at line {}: {}", i, line.trim());
            break;
        }
    }
    
    assert!(call_line.is_some(), "Should find Redo callback in Chisel.txt");
    let line_num = call_line.unwrap();
    let line_text = lines[line_num];
    
    // Find the position of "Redo)" at the end
    let redo_pos = line_text.rfind("Redo)").expect("Should find 'Redo)' in line");
    
    // Hover on "Redo" when used as callback
    let hover = provider.provide_hover(
        &ast,
        Position { line: line_num as u32, character: (redo_pos + 2) as u32 },
        &source,
        &scope,
    );
    
    assert!(hover.is_some(), "Should have hover for Redo callback");
    let hover = hover.unwrap();
    
    if let tower_lsp::lsp_types::HoverContents::Markup(content) = hover.contents {
        println!("Hover content: {}", content.value);
        assert!(content.value.contains("func Redo"), 
            "Should show Redo function when used as callback, got: {}", content.value);
        assert!(content.value.contains("User-defined function"), "Should indicate it's a function");
    } else {
        panic!("Expected Markup hover content");
    }
}
