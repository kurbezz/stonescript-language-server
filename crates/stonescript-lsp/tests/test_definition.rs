//! Test go-to-definition functionality

use stonescript_lsp::providers::definition::DefinitionProvider;
use stonescript_lsp::utils::ScopeAnalyzer;
use stonescript_parser::parse_source;
use tower_lsp::lsp_types::{GotoDefinitionResponse, Position, Url};

#[test]
fn test_goto_definition_for_function() {
    let source = r#"func MakePanel(parent, x, y)
  return ui.AddPanel()

var panel = MakePanel(null, 0, 0)
"#;

    let ast = parse_source(source).expect("Failed to parse");
    let mut scope = ScopeAnalyzer::new();
    scope.analyze_ast(&ast);

    let provider = DefinitionProvider::new();
    let uri = Url::parse("file:///test.txt").unwrap();

    // Click on "MakePanel" in function call at line 3
    let definition = provider.provide_definition(
        &ast,
        Position {
            line: 3,
            character: 15,
        },
        source,
        &scope,
        &uri,
    );

    assert!(definition.is_some(), "Should find definition for function");

    if let Some(GotoDefinitionResponse::Scalar(location)) = definition {
        // Should point to line 0 where function is defined
        assert_eq!(
            location.range.start.line, 0,
            "Should point to function definition line"
        );
        assert_eq!(location.uri, uri, "Should point to same file");
    } else {
        panic!("Expected scalar location response");
    }
}

#[test]
fn test_goto_definition_for_variable() {
    let source = r#"var x = 10
var y = x + 5
"#;

    let ast = parse_source(source).expect("Failed to parse");
    let mut scope = ScopeAnalyzer::new();
    scope.analyze_ast(&ast);

    let provider = DefinitionProvider::new();
    let uri = Url::parse("file:///test.txt").unwrap();

    // Click on "x" in second line
    let definition = provider.provide_definition(
        &ast,
        Position {
            line: 1,
            character: 8,
        },
        source,
        &scope,
        &uri,
    );

    assert!(definition.is_some(), "Should find definition for variable");

    if let Some(GotoDefinitionResponse::Scalar(location)) = definition {
        // Should point to line 0 where x is defined
        assert_eq!(
            location.range.start.line, 0,
            "Should point to variable definition line"
        );
    } else {
        panic!("Expected scalar location response");
    }
}

#[test]
fn test_goto_definition_multiple_functions() {
    let source = r#"func First()
  return 1

func Second()
  return 2

func Third()
  return First() + Second()
"#;

    let ast = parse_source(source).expect("Failed to parse");
    let mut scope = ScopeAnalyzer::new();
    scope.analyze_ast(&ast);

    let provider = DefinitionProvider::new();
    let uri = Url::parse("file:///test.txt").unwrap();

    // Click on "First" in Third function
    let definition = provider.provide_definition(
        &ast,
        Position {
            line: 7,
            character: 10,
        },
        source,
        &scope,
        &uri,
    );

    assert!(definition.is_some(), "Should find definition for First");

    if let Some(GotoDefinitionResponse::Scalar(location)) = definition {
        assert_eq!(
            location.range.start.line, 0,
            "Should point to First function"
        );
    } else {
        panic!("Expected scalar location response");
    }

    // Click on "Second" in Third function
    let definition = provider.provide_definition(
        &ast,
        Position {
            line: 7,
            character: 21,
        },
        source,
        &scope,
        &uri,
    );

    assert!(definition.is_some(), "Should find definition for Second");

    if let Some(GotoDefinitionResponse::Scalar(location)) = definition {
        assert_eq!(
            location.range.start.line, 3,
            "Should point to Second function"
        );
    } else {
        panic!("Expected scalar location response");
    }
}

#[test]
fn test_goto_definition_with_chisel_file() {
    // Load the actual Chisel.txt file
    let source = std::fs::read_to_string("../../test_scripts/Chisel.txt")
        .expect("Failed to read Chisel.txt");

    let ast = parse_source(&source).expect("Failed to parse Chisel.txt");
    let mut scope = ScopeAnalyzer::new();
    scope.analyze_ast(&ast);

    let provider = DefinitionProvider::new();
    let uri = Url::parse("file:///Chisel.txt").unwrap();

    // Find line with "func MakePanel" definition
    let lines: Vec<&str> = source.lines().collect();
    let mut make_panel_def_line = None;
    for (i, line) in lines.iter().enumerate() {
        if line.contains("func MakePanel") {
            make_panel_def_line = Some(i);
            break;
        }
    }

    assert!(
        make_panel_def_line.is_some(),
        "Should find MakePanel function definition"
    );
    let def_line = make_panel_def_line.unwrap();

    // Find a line where MakePanel is called (should be later in the file)
    let mut make_panel_call_line = None;
    for (i, line) in lines.iter().enumerate().skip(def_line + 1) {
        if line.contains("MakePanel(") && !line.contains("func MakePanel") {
            make_panel_call_line = Some(i);
            break;
        }
    }

    assert!(
        make_panel_call_line.is_some(),
        "Should find MakePanel function call"
    );
    let call_line = make_panel_call_line.unwrap();

    // Find column position of "MakePanel" in the call
    let call_line_text = lines[call_line];
    let col = call_line_text
        .find("MakePanel")
        .expect("Should find MakePanel in line");

    // Click on MakePanel call
    let definition = provider.provide_definition(
        &ast,
        Position {
            line: call_line as u32,
            character: (col + 5) as u32,
        },
        &source,
        &scope,
        &uri,
    );

    assert!(
        definition.is_some(),
        "Should find definition for MakePanel call"
    );

    if let Some(GotoDefinitionResponse::Scalar(location)) = definition {
        assert_eq!(location.range.start.line, def_line as u32, "Should point to MakePanel definition");
    } else {
        panic!("Expected scalar location response");
    }
}

#[test]
fn test_goto_definition_for_var_declaration() {
    let source = r#"var initialized = false
var basePanel = null

?initialized
  basePanel = ui.AddPanel()
"#;

    let ast = parse_source(source).expect("Failed to parse");
    let mut scope = ScopeAnalyzer::new();
    scope.analyze_ast(&ast);

    let provider = DefinitionProvider::new();
    let uri = Url::parse("file:///test.txt").unwrap();

    // Click on "initialized" in condition
    let definition = provider.provide_definition(
        &ast,
        Position {
            line: 3,
            character: 2,
        },
        source,
        &scope,
        &uri,
    );

    assert!(
        definition.is_some(),
        "Should find definition for initialized"
    );

    if let Some(GotoDefinitionResponse::Scalar(location)) = definition {
        assert_eq!(
            location.range.start.line, 0,
            "Should point to initialized declaration"
        );
    } else {
        panic!("Expected scalar location response");
    }

    // Click on "basePanel" in assignment
    let definition = provider.provide_definition(
        &ast,
        Position {
            line: 4,
            character: 3,
        },
        source,
        &scope,
        &uri,
    );

    assert!(definition.is_some(), "Should find definition for basePanel");

    if let Some(GotoDefinitionResponse::Scalar(location)) = definition {
        assert_eq!(
            location.range.start.line, 1,
            "Should point to basePanel declaration"
        );
    } else {
        panic!("Expected scalar location response");
    }
}

#[test]
fn test_goto_definition_nested_function_calls() {
    let source = r#"func Inner()
  return 42

func Outer()
  return Inner()

var result = Outer()
"#;

    let ast = parse_source(source).expect("Failed to parse");
    let mut scope = ScopeAnalyzer::new();
    scope.analyze_ast(&ast);

    let provider = DefinitionProvider::new();
    let uri = Url::parse("file:///test.txt").unwrap();

    // Click on "Inner" inside Outer function
    let definition = provider.provide_definition(
        &ast,
        Position {
            line: 4,
            character: 10,
        },
        source,
        &scope,
        &uri,
    );

    assert!(definition.is_some(), "Should find definition for Inner");

    if let Some(GotoDefinitionResponse::Scalar(location)) = definition {
        assert_eq!(
            location.range.start.line, 0,
            "Should point to Inner function"
        );
    } else {
        panic!("Expected scalar location response");
    }

    // Click on "Outer" in variable assignment
    let definition = provider.provide_definition(
        &ast,
        Position {
            line: 6,
            character: 15,
        },
        source,
        &scope,
        &uri,
    );

    assert!(definition.is_some(), "Should find definition for Outer");

    if let Some(GotoDefinitionResponse::Scalar(location)) = definition {
        assert_eq!(
            location.range.start.line, 3,
            "Should point to Outer function"
        );
    } else {
        panic!("Expected scalar location response");
    }
}
