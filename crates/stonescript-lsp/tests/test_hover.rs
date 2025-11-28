//! Test hover functionality

use stonescript_lsp::providers::hover::HoverProvider;
use stonescript_lsp::utils::ScopeAnalyzer;
use stonescript_parser::parse_source;
use tower_lsp::lsp_types::Position;

#[test]
fn test_hover_on_user_defined_function() {
    let source = r#"func MakePanel(parent, x, y, w, h)
  var panel = ui.AddPanel()
  return panel

var myPanel = MakePanel(null, 0, 0, 100, 100)
"#;

    let ast = parse_source(source).expect("Failed to parse");
    let mut scope = ScopeAnalyzer::new();
    scope.analyze_ast(&ast);
    
    let provider = HoverProvider::new();
    
    // Hover on function call "MakePanel" at line 4, column 15
    let hover = provider.provide_hover(
        &ast,
        Position { line: 4, character: 15 },
        source,
        &scope,
    );
    
    assert!(hover.is_some(), "Should have hover for function");
    let hover = hover.unwrap();
    
    // Check that it shows function signature
    if let tower_lsp::lsp_types::HoverContents::Markup(content) = hover.contents {
        assert!(content.value.contains("func MakePanel"), "Should show function name");
        assert!(content.value.contains("parent"), "Should show parameters");
        assert!(content.value.contains("User-defined function"), "Should indicate it's user-defined");
    } else {
        panic!("Expected Markup hover content");
    }
}

#[test]
fn test_hover_on_variable() {
    let source = r#"var x = 10
var y = x + 5
"#;

    let ast = parse_source(source).expect("Failed to parse");
    let mut scope = ScopeAnalyzer::new();
    scope.analyze_ast(&ast);
    
    let provider = HoverProvider::new();
    
    // Hover on "x" in second line
    let hover = provider.provide_hover(
        &ast,
        Position { line: 1, character: 8 },
        source,
        &scope,
    );
    
    assert!(hover.is_some(), "Should have hover for variable");
    let hover = hover.unwrap();
    
    if let tower_lsp::lsp_types::HoverContents::Markup(content) = hover.contents {
        assert!(content.value.contains("var x"), "Should show variable name");
        assert!(content.value.contains("Variable in scope"), "Should indicate it's a variable");
    } else {
        panic!("Expected Markup hover content");
    }
}

#[test]
fn test_hover_on_function_definition() {
    let source = r#"func TestFunction(param1, param2)
  var result = param1 + param2
  return result
"#;

    let ast = parse_source(source).expect("Failed to parse");
    let mut scope = ScopeAnalyzer::new();
    scope.analyze_ast(&ast);
    
    let provider = HoverProvider::new();
    
    // Hover on function name in definition at line 0, column 5
    let hover = provider.provide_hover(
        &ast,
        Position { line: 0, character: 5 },
        source,
        &scope,
    );
    
    assert!(hover.is_some(), "Should have hover for function definition");
    let hover = hover.unwrap();
    
    if let tower_lsp::lsp_types::HoverContents::Markup(content) = hover.contents {
        assert!(content.value.contains("func TestFunction"), "Should show function signature");
        assert!(content.value.contains("param1"), "Should show first parameter");
        assert!(content.value.contains("param2"), "Should show second parameter");
    } else {
        panic!("Expected Markup hover content");
    }
}

#[test]
fn test_hover_with_chisel_file() {
    // Load the actual Chisel.txt file
    let source = std::fs::read_to_string("../../test_scripts/Chisel.txt")
        .expect("Failed to read Chisel.txt");
    
    let ast = parse_source(&source).expect("Failed to parse Chisel.txt");
    let mut scope = ScopeAnalyzer::new();
    scope.analyze_ast(&ast);
    
    let provider = HoverProvider::new();
    
    // Find line with "func MakePanel" (should be around line 22)
    let lines: Vec<&str> = source.lines().collect();
    let mut make_panel_line = None;
    for (i, line) in lines.iter().enumerate() {
        if line.contains("func MakePanel") {
            make_panel_line = Some(i);
            break;
        }
    }
    
    assert!(make_panel_line.is_some(), "Should find MakePanel function in Chisel.txt");
    
    let line_num = make_panel_line.unwrap();
    
    // Hover on "MakePanel" in the function definition
    let hover = provider.provide_hover(
        &ast,
        Position { line: line_num as u32, character: 5 },
        &source,
        &scope,
    );
    
    assert!(hover.is_some(), "Should have hover for MakePanel");
    let hover = hover.unwrap();
    
    if let tower_lsp::lsp_types::HoverContents::Markup(content) = hover.contents {
        assert!(content.value.contains("func MakePanel"), "Should show MakePanel signature");
        assert!(content.value.contains("parent"), "Should show parent parameter");
    } else {
        panic!("Expected Markup hover content");
    }
}

#[test]
fn test_hover_on_function_call_vs_variable() {
    let source = r#"var BackgroundFade = null

func OpenSaveScreen()
  BackgroundFade = ui.AddCanvas()

var result = OpenSaveScreen()
"#;

    let ast = parse_source(source).expect("Failed to parse");
    let mut scope = ScopeAnalyzer::new();
    scope.analyze_ast(&ast);
    
    let provider = HoverProvider::new();
    
    // Hover on function name in definition at line 2
    let hover_on_func_def = provider.provide_hover(
        &ast,
        Position { line: 2, character: 7 },
        source,
        &scope,
    );
    
    assert!(hover_on_func_def.is_some(), "Should have hover for function definition");
    let hover = hover_on_func_def.unwrap();
    
    if let tower_lsp::lsp_types::HoverContents::Markup(content) = hover.contents {
        assert!(content.value.contains("func OpenSaveScreen"), "Should show function signature");
        assert!(content.value.contains("User-defined function"), "Should indicate it's a function");
    } else {
        panic!("Expected Markup hover content");
    }
    
    // Hover on variable "BackgroundFade" at line 0
    let hover_on_var = provider.provide_hover(
        &ast,
        Position { line: 0, character: 8 },
        source,
        &scope,
    );
    
    assert!(hover_on_var.is_some(), "Should have hover for variable");
    let hover = hover_on_var.unwrap();
    
    if let tower_lsp::lsp_types::HoverContents::Markup(content) = hover.contents {
        assert!(content.value.contains("var BackgroundFade"), "Should show variable");
        assert!(content.value.contains("Variable in scope"), "Should indicate it's a variable");
    } else {
        panic!("Expected Markup hover content");
    }
}
