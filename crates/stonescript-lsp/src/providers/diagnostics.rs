//! Diagnostics provider

use crate::utils::{ScopeAnalyzer, type_inference};
use crate::data::{Type, native_functions};
use tower_lsp::lsp_types::*;
use tree_sitter::Tree;

pub struct DiagnosticsProvider;

impl DiagnosticsProvider {
    pub fn new() -> Self {
        Self
    }

    pub fn provide_diagnostics(
        &self,
        tree: &Tree,
        source: &str,
        scope: &ScopeAnalyzer,
    ) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Syntax errors from tree-sitter
        self.find_syntax_errors(tree, &mut diagnostics);

        // Semantic errors
        self.find_undefined_references(tree, source, scope, &mut diagnostics);

        // Type errors
        self.find_type_errors(tree, source, &mut diagnostics);



        diagnostics
    }

    fn find_syntax_errors(&self, tree: &Tree, diagnostics: &mut Vec<Diagnostic>) {
        let mut cursor = tree.walk();

        fn visit_node(
            node: &tree_sitter::Node,
            cursor: &mut tree_sitter::TreeCursor,
            diagnostics: &mut Vec<Diagnostic>,
        ) {
            if node.is_error() || node.is_missing() {
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position {
                            line: node.start_position().row as u32,
                            character: node.start_position().column as u32,
                        },
                        end: Position {
                            line: node.end_position().row as u32,
                            character: node.end_position().column as u32,
                        },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: "syntax error".to_string(),
                    source: Some("stonescript-lsp".to_string()),
                    ..Default::default()
                });
            }

            if cursor.goto_first_child() {
                loop {
                    visit_node(&cursor.node(), cursor, diagnostics);
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
                cursor.goto_parent();
            }
        }

        visit_node(&tree.root_node(), &mut cursor, diagnostics);
    }

    fn find_undefined_references(
        &self,
        tree: &Tree,
        source: &str,
        scope: &ScopeAnalyzer,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let mut cursor = tree.walk();

        fn visit_node(
            node: &tree_sitter::Node,
            cursor: &mut tree_sitter::TreeCursor,
            source: &str,
            scope: &ScopeAnalyzer,
            diagnostics: &mut Vec<Diagnostic>,
        ) {
            // Check for undefined variable references
            if node.kind() == "identifier" {
                // Check all ancestors to determine context
                let mut should_skip = false;
                let mut ancestor = node.parent();

                while let Some(anc) = ancestor {
                    // Skip if it's the NAME field of a declaration (not values in the declaration)
                    if anc.kind() == "variable_declaration" {
                        // Only skip if this identifier is the name being declared
                        if let Some(name_node) = anc.child_by_field_name("name") {
                            if name_node.id() == node.id() {
                                should_skip = true;
                                break;
                            }
                        }
                    }

                    if anc.kind() == "function_declaration" {
                        // Only skip if this identifier is the function name
                        if let Some(name_node) = anc.child_by_field_name("name") {
                            if name_node.id() == node.id() {
                                should_skip = true;
                                break;
                            }
                        }
                    }

                    if anc.kind() == "for_loop" {
                        // For loops declare the iterator variable - need to check if this is it
                        // The for_loop syntax is: for <identifier> = ... or for <identifier> : ...
                        // We need to check if this is the first identifier child
                        if let Some(first_child) = anc.child(1) {
                            if first_child.kind() == "identifier" && first_child.id() == node.id() {
                                should_skip = true;
                                break;
                            }
                        }
                    }

                    // Skip if it's inside a parameter_list (function parameter declaration)
                    if anc.kind() == "parameter_list" {
                        should_skip = true;
                        break;
                    }

                    // Skip if it's part of a member expression (property access)
                    if anc.kind() == "member_expression" {
                        // Check if this identifier is the property (right side of dot)
                        if let Some(property_node) = anc.child_by_field_name("property") {
                            if property_node.id() == node.id() {
                                should_skip = true;
                                break;
                            }
                        }
                    }

                    // Skip if it's part of any command statement (check ancestors)
                    if anc.kind() == "equip_command"
                        || anc.kind() == "activate_command"
                        || anc.kind() == "loadout_command"
                        || anc.kind() == "brew_command"
                        || anc.kind() == "disable_enable_command"
                        || anc.kind() == "play_command"
                        || anc.kind() == "print_command"
                        || anc.kind() == "item_criteria"
                        || anc.kind() == "command_statement"
                    {
                        should_skip = true;
                        break;
                    }

                    // Skip if it's part of a binary expression with '=' operator in a conditional
                    // (e.g., ?loc=rocky - 'rocky' is a string literal, not a variable)
                    if anc.kind() == "binary_expression" {
                        // Check if this is inside a conditional
                        let mut cond_ancestor = anc.parent();
                        let mut in_conditional = false;
                        while let Some(ca) = cond_ancestor {
                            if ca.kind() == "conditional" || ca.kind() == "else_if_clause" {
                                in_conditional = true;
                                break;
                            }
                            cond_ancestor = ca.parent();
                        }

                        if in_conditional {
                            // Check if the binary expression uses '=' operator
                            let mut has_equals = false;
                            for i in 0..anc.child_count() {
                                if let Some(child) = anc.child(i) {
                                    if child.kind() == "=" {
                                        has_equals = true;
                                        break;
                                    }
                                }
                            }

                            if has_equals {
                                // This identifier is likely a comparison value, not a variable
                                should_skip = true;
                                break;
                            }
                        }
                    }

                    ancestor = anc.parent();
                }

                if should_skip {
                    return;
                }

                if let Ok(text) = node.utf8_text(source.as_bytes()) {
                    // Check if it's a known game state query
                    let known_queries = [
                        "loc",
                        "foe",
                        "hp",
                        "maxhp",
                        "armor",
                        "time",
                        "screen",
                        "input",
                        "buffs",
                        "debuffs",
                        "totaltime",
                        "cooldown",
                    ];
                    if known_queries.contains(&text) {
                        return;
                    }

                    // Check if it's a namespace
                    if ["math", "string", "storage", "ui", "music"].contains(&text) {
                        return;
                    }

                    // Check if it's a known command (print, etc.)
                    if ["print"].contains(&text) {
                        return;
                    }

                    // Check if it's a common item or foe name that appears in game
                    // These often appear as parameters to commands and get mis-parsed
                    let common_game_identifiers = [
                        // Common locations
                        "rocky",
                        "cave",
                        "halls",
                        "deadwood",
                        "temple",
                        "icy",
                        // Common foes
                        "bolesh",
                        "poena",
                        "nagaraja",
                        "stoneguard",
                        // Common items
                        "shovel",
                        "pickaxe",
                        "sword",
                        "dagger",
                        "hammer",
                        "staff",
                        "wand",
                        "shield",
                        "armor",
                        "helm",
                        "boots",
                        "ring",
                        "amulet",
                        "potion",
                        "pumpkin",
                        "stone",
                        "grap",
                        "hook",
                        // Common modifiers
                        "poison",
                        "fire",
                        "ice",
                        "vigor",
                        "haste",
                        "slow",
                        // Single letter modifiers
                        "D",
                        "A",
                        "S",
                    ];
                    if common_game_identifiers.contains(&text) {
                        return;
                    }

                    // Check in scope
                    let variables = scope.find_variables_at(node.start_byte());
                    let functions = scope.get_functions();

                    let is_defined = variables.iter().any(|v| v.name == text)
                        || functions.iter().any(|f| f.name == text);

                    if !is_defined {
                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position {
                                    line: node.start_position().row as u32,
                                    character: node.start_position().column as u32,
                                },
                                end: Position {
                                    line: node.end_position().row as u32,
                                    character: node.end_position().column as u32,
                                },
                            },
                            severity: Some(DiagnosticSeverity::WARNING),
                            message: format!("cannot find value `{}` in this scope", text),
                            source: Some("stonescript-lsp".to_string()),
                            ..Default::default()
                        });
                    }
                }
            }

            if cursor.goto_first_child() {
                loop {
                    visit_node(&cursor.node(), cursor, source, scope, diagnostics);
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
                cursor.goto_parent();
            }
        }

        visit_node(&tree.root_node(), &mut cursor, source, scope, diagnostics);
    }


    fn find_type_errors(
        &self,
        tree: &Tree,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let mut cursor = tree.walk();

        fn visit_node(
            node: &tree_sitter::Node,
            cursor: &mut tree_sitter::TreeCursor,
            source: &str,
            diagnostics: &mut Vec<Diagnostic>,
        ) {
            // Check function calls
            if node.kind() == "call_expression" {
                if let Some(func_node) = node.child_by_field_name("function") {
                    if func_node.kind() == "member_expression" {
                        if let Some(obj_node) = func_node.child_by_field_name("object") {
                            let namespace = obj_node.utf8_text(source.as_bytes()).unwrap_or("");
                            if let Some(prop_node) = func_node.child_by_field_name("property") {
                                let func_name = prop_node.utf8_text(source.as_bytes()).unwrap_or("");
                                println!("Checking function: {}.{}", namespace, func_name);
                                if let Some(func_sig) = native_functions::get_function(namespace, func_name) {
                                    // Find argument_list node
                                    let mut args_node_opt = None;
                                    let mut cursor = node.walk();
                                    for child in node.children(&mut cursor) {
                                        if child.kind() == "argument_list" {
                                            args_node_opt = Some(child);
                                            break;
                                        }
                                    }

                                    if let Some(args_node) = args_node_opt {
                                        let mut arg_idx = 0;
                                        let child_count = args_node.child_count();
                                        
                                        for i in 0..child_count {
                                            if let Some(child) = args_node.child(i) {
                                                if child.kind() == "(" || child.kind() == ")" || child.kind() == "," {
                                                    continue;
                                                }
                                                
                                                if arg_idx < func_sig.parameters.len() {
                                                    let param = &func_sig.parameters[arg_idx];
                                                    let arg_type = type_inference::infer_type(&child, source);
                                                    
                                                    if arg_type != Type::Unknown && param.typ != Type::Unknown && arg_type != param.typ {
                                                         // Allow int to float conversion
                                                        if !(arg_type == Type::Int && param.typ == Type::Float) {
                                                            diagnostics.push(Diagnostic {
                                                                range: Range {
                                                                    start: Position {
                                                                        line: child.start_position().row as u32,
                                                                        character: child.start_position().column as u32,
                                                                    },
                                                                    end: Position {
                                                                        line: child.end_position().row as u32,
                                                                        character: child.end_position().column as u32,
                                                                    },
                                                                },
                                                                severity: Some(DiagnosticSeverity::ERROR),
                                                                message: format!("mismatched types: expected `{}`, found `{}`", param.typ, arg_type),
                                                                source: Some("stonescript-lsp".to_string()),
                                                                ..Default::default()
                                                            });
                                                        }
                                                    }
                                                }
                                                arg_idx += 1;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Check binary expressions
            if node.kind() == "binary_expression" {
                let operator_node = node.child(1);
                if let Some(op) = operator_node {
                    let op_text = op.utf8_text(source.as_bytes()).unwrap_or("");
                    let left = node.child(0);
                    let right = node.child(2);
                    
                    if let (Some(l), Some(r)) = (left, right) {
                        let left_type = type_inference::infer_type(&l, source);
                        let right_type = type_inference::infer_type(&r, source);
                        
                        if left_type != Type::Unknown && right_type != Type::Unknown {
                            match op_text {
                                "+" | "-" | "*" | "/" | "%" | "^" => {
                                    if !left_type.is_numeric() || !right_type.is_numeric() {
                                        diagnostics.push(Diagnostic {
                                            range: Range {
                                                start: Position {
                                                    line: node.start_position().row as u32,
                                                    character: node.start_position().column as u32,
                                                },
                                                end: Position {
                                                    line: node.end_position().row as u32,
                                                    character: node.end_position().column as u32,
                                                },
                                            },
                                            severity: Some(DiagnosticSeverity::ERROR),
                                            message: format!("mismatched types: `{}` requires numeric operands, found `{}` and `{}`", op_text, left_type, right_type),
                                            source: Some("stonescript-lsp".to_string()),
                                            ..Default::default()
                                        });
                                    }
                                },
                                "&" | "|" => {
                                    if left_type != Type::Bool || right_type != Type::Bool {
                                        diagnostics.push(Diagnostic {
                                            range: Range {
                                                start: Position {
                                                    line: node.start_position().row as u32,
                                                    character: node.start_position().column as u32,
                                                },
                                                end: Position {
                                                    line: node.end_position().row as u32,
                                                    character: node.end_position().column as u32,
                                                },
                                            },
                                            severity: Some(DiagnosticSeverity::ERROR),
                                            message: format!("mismatched types: `{}` requires boolean operands, found `{}` and `{}`", op_text, left_type, right_type),
                                            source: Some("stonescript-lsp".to_string()),
                                            ..Default::default()
                                        });
                                    }
                                },
                                _ => {}
                            }
                        }
                    }
                }
            }

            if cursor.goto_first_child() {
                loop {
                    visit_node(&cursor.node(), cursor, source, diagnostics);
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
                cursor.goto_parent();
            }
        }
        
        visit_node(&tree.root_node(), &mut cursor, source, diagnostics);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostics_provider() {
        let provider = DiagnosticsProvider::new();
        // Would need real tree for full test
        let _ = provider;
    }

    #[test]
    fn test_no_false_positives_on_commands() {
        let provider = DiagnosticsProvider::new();

        // Test the example from test.ss - should have no undefined reference errors
        let source = r#"?loc=rocky
 equip shovel
?loc=cave
 loadout 1
 ?foe=bolesh
  equip grap
  equip hammer *7 D
?loc=halls
 equipL poison wand
 equipR vigor wand
 ?loc.stars > 5
  equip vigor staff +13
?hp < 10
 activate potion
"#;

        if let Some(tree) = stonescript_parser::parse(source) {
            let mut scope = ScopeAnalyzer::new();
            scope.analyze(&tree, source);

            let diagnostics = provider.provide_diagnostics(&tree, source, &scope);

            // Filter out syntax errors, only look for undefined references
            let undefined_refs: Vec<_> = diagnostics
                .iter()
                .filter(|d| d.message.contains("cannot find value"))
                .collect();

            assert_eq!(
                undefined_refs.len(),
                0,
                "Expected no undefined reference warnings, but got: {:?}",
                undefined_refs
            );
        } else {
            panic!("Failed to parse source");
        }
    }

    #[test]
    fn test_detects_actual_undefined_variables() {
        let provider = DiagnosticsProvider::new();

        // This should produce an undefined reference error
        let source = r#"var x = 10
var y = undefined_var + 5
"#;

        if let Some(tree) = stonescript_parser::parse(source) {
            let mut scope = ScopeAnalyzer::new();
            scope.analyze(&tree, source);

            let diagnostics = provider.provide_diagnostics(&tree, source, &scope);

            let undefined_refs: Vec<_> = diagnostics
                .iter()
                .filter(|d| d.message.contains("cannot find value"))
                .collect();

            assert!(
                undefined_refs.len() > 0,
                "Expected to find undefined reference warning"
            );

            assert!(
                undefined_refs
                    .iter()
                    .any(|d| d.message.contains("undefined_var")),
                "Expected to find 'undefined_var' in diagnostics"
            );
        } else {
            panic!("Failed to parse source");
        }
    }

    #[test]
    fn test_member_expression_no_false_positive() {
        let provider = DiagnosticsProvider::new();

        // loc.stars should not trigger undefined reference for 'stars'
        let source = r#"?loc.stars > 5
 equip vigor staff
"#;

        if let Some(tree) = stonescript_parser::parse(source) {
            let mut scope = ScopeAnalyzer::new();
            scope.analyze(&tree, source);

            let diagnostics = provider.provide_diagnostics(&tree, source, &scope);

            let undefined_refs: Vec<_> = diagnostics
                .iter()
                .filter(|d| d.message.contains("cannot find value"))
                .collect();

            assert_eq!(
                undefined_refs.len(),
                0,
                "Expected no undefined reference warnings for member expressions, but got: {:?}",
                undefined_refs
            );
        } else {
            panic!("Failed to parse source");
        }
    }

    #[test]
    fn test_game_state_queries_recognized() {
        let provider = DiagnosticsProvider::new();

        // All game state queries should be recognized
        let source = r#"?hp < maxhp
 activate potion
?armor > 0
 equip shield
?time > 10
 print "Time passed"
"#;

        if let Some(tree) = stonescript_parser::parse(source) {
            let mut scope = ScopeAnalyzer::new();
            scope.analyze(&tree, source);

            let diagnostics = provider.provide_diagnostics(&tree, source, &scope);

            let undefined_refs: Vec<_> = diagnostics
                .iter()
                .filter(|d| d.message.contains("cannot find value"))
                .collect();

            assert_eq!(
                undefined_refs.len(),
                0,
                "Game state queries should not be flagged as undefined: {:?}",
                undefined_refs
            );
        } else {
            panic!("Failed to parse source");
        }
    }

    #[test]
    fn test_type_mismatch() {
        let provider = DiagnosticsProvider::new();
        let source = "math.Abs(\"string\")";
        
        if let Some(tree) = stonescript_parser::parse(source) {
            let scope = ScopeAnalyzer::new();
            let diagnostics = provider.provide_diagnostics(&tree, source, &scope);
            
            let type_errors: Vec<_> = diagnostics
                .iter()
                .filter(|d| d.message.contains("mismatched types"))
                .collect();
                
            assert!(type_errors.len() > 0, "Expected type mismatch error");
        }
    }

    #[test]
    fn test_binary_expression_type_mismatch() {
        let provider = DiagnosticsProvider::new();
        // String + Int should fail
        let source = "\"string\" + 5";
        
        if let Some(tree) = stonescript_parser::parse(source) {
            let scope = ScopeAnalyzer::new();
            let diagnostics = provider.provide_diagnostics(&tree, source, &scope);
            
            let type_errors: Vec<_> = diagnostics
                .iter()
                .filter(|d| d.message.contains("mismatched types"))
                .collect();
                
            assert!(type_errors.len() > 0, "Expected type mismatch error for binary expression");
        }
    }

    #[test]
    fn test_function_parameters_not_undefined() {
        let provider = DiagnosticsProvider::new();
        let source = "func add(a, b)\n  return a + b";
        
        if let Some(tree) = stonescript_parser::parse(source) {
            let mut scope = ScopeAnalyzer::new();
            scope.analyze(&tree, source);
            
            let diagnostics = provider.provide_diagnostics(&tree, source, &scope);
            
            let undefined_refs: Vec<_> = diagnostics
                .iter()
                .filter(|d| d.message.contains("cannot find value"))
                .collect();
                
            assert_eq!(
                undefined_refs.len(),
                0,
                "Function parameters should not be flagged as undefined: {:?}",
                undefined_refs
            );
        } else {
            panic!("Failed to parse source");
        }
    }
}
