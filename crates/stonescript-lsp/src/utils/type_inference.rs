//! Type inference from tree-sitter AST

use tree_sitter::Node;
use crate::data::{Type, native_functions, game_state};
use crate::utils::ScopeAnalyzer;

/// Infer type from an AST node
pub fn infer_type(node: &Node, source: &str) -> Type {
    infer_type_with_scope(node, source, None, None)
}

/// Infer type from an AST node with scope information
pub fn infer_type_with_scope(node: &Node, source: &str, scope: Option<&ScopeAnalyzer>, tree: Option<&tree_sitter::Tree>) -> Type {
    match node.kind() {
        "number" => Type::Int,
        "float" => Type::Float,
        "string" => Type::String,
        "boolean" => Type::Bool,
        "null" => Type::Unknown,
        
        "array" => {
            // For now, return generic array type
            // Full type inference would need more context
            Type::Unknown  // Arrays need runtime analysis
        }
        
        "member_expression" => {
            // Try to infer from object
            if let Some(object_node) = node.child_by_field_name("object") {
                let object_text = object_node.utf8_text(source.as_bytes()).unwrap_or("");
                
                // Game state objects
                if let Some(query) = game_state::get_game_state(object_text) {
                    if let Some(prop_node) = node.child_by_field_name("property") {
                        let prop_name = prop_node.utf8_text(source.as_bytes()).unwrap_or("");
                        if let Some(properties) = query.properties {
                            if let Some(prop) = properties.iter().find(|p| p.name == prop_name) {
                                return prop.typ.clone();
                            }
                        }
                    }
                }

                // Native namespaces
                match object_text {
                    "math" => return Type::Float, // Math functions return numbers
                    "string" => return Type::String, // String functions return strings (mostly)
                    _ => {}
                }
            }
            Type::Unknown
        }
        
        "call_expression" => {
            // Function calls - try to infer from function name
            if let Some(func_node) = node.child_by_field_name("function") {
                if func_node.kind() == "member_expression" {
                    // Namespace function call like math.Sqrt()
                    if let Some(obj_node) = func_node.child_by_field_name("object") {
                        let namespace = obj_node.utf8_text(source.as_bytes()).unwrap_or("");
                        if let Some(prop_node) = func_node.child_by_field_name("property") {
                            let func_name = prop_node.utf8_text(source.as_bytes()).unwrap_or("");
                            if let Some(func) = native_functions::get_function(namespace, func_name) {
                                return func.return_type.clone();
                            }
                        }
                    }
                } else if func_node.kind() == "identifier" {
                    // User-defined function call
                    let func_name = func_node.utf8_text(source.as_bytes()).unwrap_or("");
                    
                    // Look up the function in the scope analyzer
                    if let Some(scope_analyzer) = scope {
                        if let Some(user_func) = scope_analyzer.get_function(func_name) {
                            // If the function already has a cached return type, use it
                            if let Some(return_type) = &user_func.return_type {
                                return return_type.clone();
                            }
                            
                            // Otherwise try to infer from the function body
                            if let (Some(body_start), Some(body_end), Some(tree_ref)) = 
                                (user_func.body_start_byte, user_func.body_end_byte, tree) {
                                if let Some(inferred_type) = infer_function_return_type(
                                    tree_ref, body_start, body_end, source, scope
                                ) {
                                    return inferred_type;
                                }
                            }
                        }
                    }
                }
            }
            Type::Unknown
        }
        
        "binary_expression" => {
            // Binary operations typically return numbers or booleans
            let operator_node = node.child(1);
            if let Some(op) = operator_node {
                let op_text = op.utf8_text(source.as_bytes()).unwrap_or("");
                match op_text {
                    "+" | "-" | "*" | "/" | "%" | "^" => {
                        // Check operands for float vs int
                        let left = node.child(0).map(|n| infer_type_with_scope(&n, source, scope, tree)).unwrap_or(Type::Unknown);
                        let right = node.child(2).map(|n| infer_type_with_scope(&n, source, scope, tree)).unwrap_or(Type::Unknown);
                        if left == Type::Float || right == Type::Float {
                            Type::Float
                        } else {
                            Type::Int
                        }
                    },
                    "=" | "!" | "<" | ">" | "<=" | ">=" | "&" | "|" => Type::Bool,
                    _ => Type::Unknown,
                }
            } else {
                Type::Unknown
            }
        }

        "unary_expression" => {
            let operator_node = node.child(0);
            if let Some(op) = operator_node {
                let op_text = op.utf8_text(source.as_bytes()).unwrap_or("");
                match op_text {
                    "!" => Type::Bool,
                    "-" => {
                        if let Some(operand) = node.child(1) {
                            infer_type_with_scope(&operand, source, scope, tree)
                        } else {
                            Type::Int
                        }
                    }
                    _ => Type::Unknown,
                }
            } else {
                Type::Unknown
            }
        }
        
        "identifier" => {
            let text = node.utf8_text(source.as_bytes()).unwrap_or("");
            
            // 1. Check if it's a known game state query
            if let Some(query) = game_state::get_game_state(text) {
                return query.return_type.clone();
            }
            
            // 2. Check if it's a variable in scope (including function parameters)
            if let Some(scope_analyzer) = scope {
                let variables = scope_analyzer.find_variables_at(node.start_byte());
                if let Some(var) = variables.iter().find(|v| v.name == text) {
                    // If we found the variable, we need to find its declaration to infer type
                    // But wait, if it's a parameter, we might not know the type yet unless we have type hints (which StoneScript doesn't seem to have strongly)
                    // OR if we are inferring a function return type, maybe we can assume parameters are Int/Float based on usage?
                    // For now, let's try to find the declaration.
                    
                    if let Some(tree_ref) = tree {
                         // Find the variable declaration node using byte range
                        let var_decl_node = tree_ref
                            .root_node()
                            .named_descendant_for_byte_range(var.start_byte, var.end_byte);
                            
                        if let Some(decl) = var_decl_node {
                             // If it's a variable declaration
                            if decl.kind() == "variable_declaration" {
                                if let Some(value_node) = decl.child_by_field_name("value") {
                                    // Avoid infinite recursion if the value refers back to the variable (unlikely in simple cases but possible)
                                    if value_node.start_byte() != node.start_byte() {
                                        return infer_type_with_scope(&value_node, source, scope, tree);
                                    }
                                }
                            }
                            // If it's a parameter, we currently return Unknown because StoneScript is dynamic
                            // But for the test case 'x * 2', 'x' is used in a binary expression.
                            // The binary expression inference handles Unknown operands by checking the other operand!
                            // So if x is Unknown, and 2 is Int, result is Int.
                        }
                    }
                }
            }
            
            Type::Unknown
        }

        "assignment_expression" => {
            if let Some(right) = node.child_by_field_name("right") {
                return infer_type_with_scope(&right, source, scope, tree);
            }
            Type::Unknown
        }

        "parenthesized_expression" => {
            if let Some(inner) = node.child(1) { // ( expr )
                return infer_type_with_scope(&inner, source, scope, tree);
            }
            Type::Unknown
        }
        
        _ => Type::Unknown,
    }
}

/// Infer the return type of a function from its body
/// Returns None if no return statements are found or if they return inconsistent types
pub fn infer_function_return_type(tree: &tree_sitter::Tree, body_start: usize, body_end: usize, source: &str, scope: Option<&ScopeAnalyzer>) -> Option<Type> {
    // Find all return statements in the function body
    let mut return_types = Vec::new();
    
    // Find the function body node
    let body_node = tree.root_node()
        .named_descendant_for_byte_range(body_start, body_end)?;
    
    collect_return_types(&body_node, source, &mut return_types, scope, Some(tree));
    
    if return_types.is_empty() {
        // No return statements found - function returns void/nothing
        return None;
    }
    
    // Check if all return types are the same
    let first_type = &return_types[0];
    if return_types.iter().all(|t| t == first_type) {
        Some(first_type.clone())
    } else {
        // Mixed return types - can't determine a single type
        Some(Type::Unknown)
    }
}

/// Recursively collect return types from return statements
fn collect_return_types(node: &tree_sitter::Node, source: &str, types: &mut Vec<Type>, scope: Option<&ScopeAnalyzer>, tree: Option<&tree_sitter::Tree>) {
    if node.kind() == "return_statement" {
        // Get the return value expression
        // Try field name "value" first, then fallback to second child (index 1)
        let value_node = node.child_by_field_name("value")
            .or_else(|| {
                // child(0) is "return" keyword
                if node.child_count() >= 2 {
                    node.child(1)
                } else {
                    None
                }
            });

        if let Some(value_node) = value_node {
            let return_type = infer_type_with_scope(&value_node, source, scope, tree);
            types.push(return_type);
        }
    } else {
        // Recursively check children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            collect_return_types(&child, source, types, scope, tree);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Literal Types ====================
    
    #[test]
    fn test_literal_int() {
        let source = "10";
        let tree = stonescript_parser::parse(source).unwrap();
        let node = tree.root_node().child(0).unwrap().child(0).unwrap();
        
        assert_eq!(node.kind(), "number");
        let typ = infer_type(&node, source);
        assert_eq!(typ, Type::Int);
    }

    #[test]
    fn test_literal_float() {
        let source = "3.14";
        let tree = stonescript_parser::parse(source).unwrap();
        let node = tree.root_node().child(0).unwrap().child(0).unwrap();
        
        assert_eq!(node.kind(), "float");
        let typ = infer_type(&node, source);
        assert_eq!(typ, Type::Float);
    }

    #[test]
    fn test_literal_string() {
        let source = "\"hello\"";
        let tree = stonescript_parser::parse(source).unwrap();
        let node = tree.root_node().child(0).unwrap().child(0).unwrap();
        
        assert_eq!(node.kind(), "string");
        let typ = infer_type(&node, source);
        assert_eq!(typ, Type::String);
    }

    #[test]
    fn test_literal_bool() {
        let source = "true";
        let tree = stonescript_parser::parse(source).unwrap();
        let node = tree.root_node().child(0).unwrap().child(0).unwrap();
        
        assert_eq!(node.kind(), "boolean");
        let typ = infer_type(&node, source);
        assert_eq!(typ, Type::Bool);
    }

    // ==================== Variable Declarations ====================
    
    #[test]
    fn test_variable_declaration_int() {
        let source = "var x = 5";
        let tree = stonescript_parser::parse(source).unwrap();
        let var_decl = tree.root_node().child(0).unwrap();
        
        assert_eq!(var_decl.kind(), "variable_declaration");
        if let Some(value_node) = var_decl.child_by_field_name("value") {
            let typ = infer_type(&value_node, source);
            assert_eq!(typ, Type::Int);
        } else {
            panic!("No value field in variable_declaration");
        }
    }

    #[test]
    fn test_variable_declaration_float() {
        let source = "var x = 5.5";
        let tree = stonescript_parser::parse(source).unwrap();
        let var_decl = tree.root_node().child(0).unwrap();
        
        assert_eq!(var_decl.kind(), "variable_declaration");
        if let Some(value_node) = var_decl.child_by_field_name("value") {
            let typ = infer_type(&value_node, source);
            assert_eq!(typ, Type::Float);
        } else {
            panic!("No value field in variable_declaration");
        }
    }

    #[test]
    fn test_variable_declaration_string() {
        let source = "var name = \"test\"";
        let tree = stonescript_parser::parse(source).unwrap();
        let var_decl = tree.root_node().child(0).unwrap();
        
        assert_eq!(var_decl.kind(), "variable_declaration");
        if let Some(value_node) = var_decl.child_by_field_name("value") {
            let typ = infer_type(&value_node, source);
            assert_eq!(typ, Type::String);
        } else {
            panic!("No value field in variable_declaration");
        }
    }

    #[test]
    fn test_variable_declaration_bool() {
        let source = "var flag = false";
        let tree = stonescript_parser::parse(source).unwrap();
        let var_decl = tree.root_node().child(0).unwrap();
        
        assert_eq!(var_decl.kind(), "variable_declaration");
        if let Some(value_node) = var_decl.child_by_field_name("value") {
            let typ = infer_type(&value_node, source);
            assert_eq!(typ, Type::Bool);
        } else {
            panic!("No value field in variable_declaration");
        }
    }

    #[test]
    fn test_variable_declaration_expression() {
        let source = "var result = 5 + 3";
        let tree = stonescript_parser::parse(source).unwrap();
        let var_decl = tree.root_node().child(0).unwrap();
        
        assert_eq!(var_decl.kind(), "variable_declaration");
        if let Some(value_node) = var_decl.child_by_field_name("value") {
            let typ = infer_type(&value_node, source);
            assert_eq!(typ, Type::Int);
        } else {
            panic!("No value field in variable_declaration");
        }
    }

    // ==================== Binary Expressions ====================
    
    #[test]
    fn test_binary_expression_int_addition() {
        let source = "5 + 5";
        let tree = stonescript_parser::parse(source).unwrap();
        let root = tree.root_node();
        let stmt = root.child(0).unwrap();
        let expr = stmt.child(0).unwrap();
        
        assert_eq!(expr.kind(), "binary_expression");
        let typ = infer_type(&expr, source);
        assert_eq!(typ, Type::Int);
    }

    #[test]
    fn test_binary_expression_float_arithmetic() {
        let source = "5.5 + 3.2";
        let tree = stonescript_parser::parse(source).unwrap();
        let root = tree.root_node();
        let stmt = root.child(0).unwrap();
        let expr = stmt.child(0).unwrap();
        
        assert_eq!(expr.kind(), "binary_expression");
        let typ = infer_type(&expr, source);
        assert_eq!(typ, Type::Float);
    }

    #[test]
    fn test_binary_expression_mixed_arithmetic() {
        let source = "5 + 3.2";
        let tree = stonescript_parser::parse(source).unwrap();
        let root = tree.root_node();
        let stmt = root.child(0).unwrap();
        let expr = stmt.child(0).unwrap();
        
        assert_eq!(expr.kind(), "binary_expression");
        let typ = infer_type(&expr, source);
        // Mixed int + float should return float
        assert_eq!(typ, Type::Float);
    }

    #[test]
    fn test_binary_expression_subtraction() {
        let source = "10 - 3";
        let tree = stonescript_parser::parse(source).unwrap();
        let root = tree.root_node();
        let stmt = root.child(0).unwrap();
        let expr = stmt.child(0).unwrap();
        
        assert_eq!(expr.kind(), "binary_expression");
        let typ = infer_type(&expr, source);
        assert_eq!(typ, Type::Int);
    }

    #[test]
    fn test_binary_expression_multiplication() {
        let source = "5 * 3";
        let tree = stonescript_parser::parse(source).unwrap();
        let root = tree.root_node();
        let stmt = root.child(0).unwrap();
        let expr = stmt.child(0).unwrap();
        
        assert_eq!(expr.kind(), "binary_expression");
        let typ = infer_type(&expr, source);
        assert_eq!(typ, Type::Int);
    }

    #[test]
    fn test_binary_expression_division() {
        let source = "10 / 2";
        let tree = stonescript_parser::parse(source).unwrap();
        let root = tree.root_node();
        let stmt = root.child(0).unwrap();
        let expr = stmt.child(0).unwrap();
        
        assert_eq!(expr.kind(), "binary_expression");
        let typ = infer_type(&expr, source);
        assert_eq!(typ, Type::Int);
    }

    #[test]
    fn test_binary_expression_comparison() {
        let source = "5 > 3";
        let tree = stonescript_parser::parse(source).unwrap();
        let root = tree.root_node();
        let stmt = root.child(0).unwrap();
        let expr = stmt.child(0).unwrap();
        
        assert_eq!(expr.kind(), "binary_expression");
        let typ = infer_type(&expr, source);
        assert_eq!(typ, Type::Bool);
    }

    #[test]
    fn test_binary_expression_equality() {
        let source = "5 = 3";
        let tree = stonescript_parser::parse(source).unwrap();
        let root = tree.root_node();
        let stmt = root.child(0).unwrap();
        let expr = stmt.child(0).unwrap();
        
        assert_eq!(expr.kind(), "binary_expression");
        let typ = infer_type(&expr, source);
        assert_eq!(typ, Type::Bool);
    }

    #[test]
    fn test_binary_expression_logical_and() {
        let source = "true & false";
        let tree = stonescript_parser::parse(source).unwrap();
        let root = tree.root_node();
        let stmt = root.child(0).unwrap();
        let expr = stmt.child(0).unwrap();
        
        assert_eq!(expr.kind(), "binary_expression");
        let typ = infer_type(&expr, source);
        assert_eq!(typ, Type::Bool);
    }

    // ==================== Unary Expressions ====================
    
    #[test]
    fn test_unary_expression_negation_int() {
        let source = "-5";
        let tree = stonescript_parser::parse(source).unwrap();
        let root = tree.root_node();
        let stmt = root.child(0).unwrap();
        let expr = stmt.child(0).unwrap();
        
        assert_eq!(expr.kind(), "unary_expression");
        let typ = infer_type(&expr, source);
        assert_eq!(typ, Type::Int);
    }

    #[test]
    fn test_unary_expression_negation_float() {
        let source = "-5.5";
        let tree = stonescript_parser::parse(source).unwrap();
        let root = tree.root_node();
        let stmt = root.child(0).unwrap();
        let expr = stmt.child(0).unwrap();
        
        assert_eq!(expr.kind(), "unary_expression");
        let typ = infer_type(&expr, source);
        assert_eq!(typ, Type::Float);
    }

    #[test]
    fn test_unary_expression_logical_not() {
        let source = "!true";
        let tree = stonescript_parser::parse(source).unwrap();
        let root = tree.root_node();
        let stmt = root.child(0).unwrap();
        let expr = stmt.child(0).unwrap();
        
        assert_eq!(expr.kind(), "unary_expression");
        let typ = infer_type(&expr, source);
        assert_eq!(typ, Type::Bool);
    }

    // ==================== Function Calls ====================
    
    #[test]
    fn test_function_call_math_abs() {
        let source = "math.Abs(5)";
        let tree = stonescript_parser::parse(source).unwrap();
        let root = tree.root_node();
        let stmt = root.child(0).unwrap();
        let call = stmt.child(0).unwrap();
        
        assert_eq!(call.kind(), "call_expression");
        let typ = infer_type(&call, source);
        assert_eq!(typ, Type::Float);
    }

    #[test]
    fn test_function_call_math_sqrt() {
        let source = "math.Sqrt(25)";
        let tree = stonescript_parser::parse(source).unwrap();
        let root = tree.root_node();
        let stmt = root.child(0).unwrap();
        let call = stmt.child(0).unwrap();
        
        assert_eq!(call.kind(), "call_expression");
        let typ = infer_type(&call, source);
        assert_eq!(typ, Type::Float);
    }

    #[test]
    fn test_function_call_string_sub() {
        let source = "string.Sub(\"test\", 0)";
        let tree = stonescript_parser::parse(source).unwrap();
        let root = tree.root_node();
        let stmt = root.child(0).unwrap();
        let call = stmt.child(0).unwrap();
        
        assert_eq!(call.kind(), "call_expression");
        let typ = infer_type(&call, source);
        assert_eq!(typ, Type::String);
    }

    // ==================== Member Expressions ====================
    
    #[test]
    fn test_member_expression_loc_name() {
        let source = "loc.name";
        let tree = stonescript_parser::parse(source).unwrap();
        let root = tree.root_node();
        let stmt = root.child(0).unwrap();
        let member = stmt.child(0).unwrap();
        
        assert_eq!(member.kind(), "member_expression");
        let typ = infer_type(&member, source);
        assert_eq!(typ, Type::String);
    }

    #[test]
    fn test_member_expression_foe_hp() {
        let source = "foe.hp";
        let tree = stonescript_parser::parse(source).unwrap();
        let root = tree.root_node();
        let stmt = root.child(0).unwrap();
        let member = stmt.child(0).unwrap();
        
        assert_eq!(member.kind(), "member_expression");
        let typ = infer_type(&member, source);
        assert_eq!(typ, Type::Int);
    }

    #[test]
    fn test_member_expression_screen_w() {
        let source = "screen.w";
        let tree = stonescript_parser::parse(source).unwrap();
        let root = tree.root_node();
        let stmt = root.child(0).unwrap();
        let member = stmt.child(0).unwrap();
        
        assert_eq!(member.kind(), "member_expression");
        let typ = infer_type(&member, source);
        assert_eq!(typ, Type::Int);
    }

    // ==================== Parenthesized Expressions ====================
    
    #[test]
    fn test_parenthesized_expression() {
        let source = "(5 + 3)";
        let tree = stonescript_parser::parse(source).unwrap();
        let root = tree.root_node();
        let stmt = root.child(0).unwrap();
        let paren = stmt.child(0).unwrap();
        
        assert_eq!(paren.kind(), "parenthesized_expression");
        let typ = infer_type(&paren, source);
        assert_eq!(typ, Type::Int);
    }

    #[test]
    fn test_parenthesized_float_expression() {
        let source = "(5.5 + 3.2)";
        let tree = stonescript_parser::parse(source).unwrap();
        let root = tree.root_node();
        let stmt = root.child(0).unwrap();
        let paren = stmt.child(0).unwrap();
        
        assert_eq!(paren.kind(), "parenthesized_expression");
        let typ = infer_type(&paren, source);
        assert_eq!(typ, Type::Float);
    }

    // ==================== Game State Queries ====================
    
    #[test]
    fn test_game_state_hp() {
        let source = "hp";
        let tree = stonescript_parser::parse(source).unwrap();
        
        // Find the identifier node
        let mut cursor = tree.walk();
        let mut target_node = None;
        for node in tree.root_node().children(&mut cursor) {
            if node.kind() == "identifier" && node.utf8_text(source.as_bytes()).unwrap() == "hp" {
                target_node = Some(node);
                break;
            }
            if node.child_count() > 0 {
                let child = node.child(0).unwrap();
                if child.kind() == "identifier" && child.utf8_text(source.as_bytes()).unwrap() == "hp" {
                    target_node = Some(child);
                    break;
                }
            }
        }
        
        if let Some(node) = target_node {
            let typ = infer_type(&node, source);
            assert_eq!(typ, Type::Int);
        } else {
            panic!("Could not find hp identifier node");
        }
    }

    #[test]
    fn test_game_state_time() {
        let source = "time";
        let tree = stonescript_parser::parse(source).unwrap();
        
        // Find the identifier node
        let mut cursor = tree.walk();
        let mut target_node = None;
        for node in tree.root_node().children(&mut cursor) {
            if node.kind() == "identifier" && node.utf8_text(source.as_bytes()).unwrap() == "time" {
                target_node = Some(node);
                break;
            }
            if node.child_count() > 0 {
                let child = node.child(0).unwrap();
                if child.kind() == "identifier" && child.utf8_text(source.as_bytes()).unwrap() == "time" {
                    target_node = Some(child);
                    break;
                }
            }
        }
        
        if let Some(node) = target_node {
            let typ = infer_type(&node, source);
            assert_eq!(typ, Type::Float);
        } else {
            panic!("Could not find time identifier node");
        }
    }

    #[test]
    fn test_game_state_music() {
        let source = "music";
        let tree = stonescript_parser::parse(source).unwrap();
        
        // Find the identifier node
        let mut cursor = tree.walk();
        let mut target_node = None;
        for node in tree.root_node().children(&mut cursor) {
            if node.kind() == "identifier" && node.utf8_text(source.as_bytes()).unwrap() == "music" {
                target_node = Some(node);
                break;
            }
            if node.child_count() > 0 {
                let child = node.child(0).unwrap();
                if child.kind() == "identifier" && child.utf8_text(source.as_bytes()).unwrap() == "music" {
                    target_node = Some(child);
                    break;
                }
            }
        }
        
        if let Some(node) = target_node {
            let typ = infer_type(&node, source);
            assert_eq!(typ, Type::String);
        } else {
            panic!("Could not find music identifier node");
        }
    }

    // ==================== Assignment Expressions ====================
    
    #[test]
    fn test_assignment_expression() {
        let source = "x = 10";
        let tree = stonescript_parser::parse(source).unwrap();
        let root = tree.root_node();
        let stmt = root.child(0).unwrap();
        let assign = stmt.child(0).unwrap();
        
        if assign.kind() == "assignment_expression" {
            let typ = infer_type(&assign, source);
            assert_eq!(typ, Type::Int);
        }
    }

    #[test]
    fn test_user_function_call_inference() {
        let source = "func test(x)\n  return x * 2\n\nvar result = test(5)";
        let tree = stonescript_parser::parse(source).unwrap();
        
        let mut analyzer = ScopeAnalyzer::new();
        analyzer.analyze(&tree, source);
        
        // Find the variable declaration for 'result'
        let root = tree.root_node();
        let var_decl = root.child(1).unwrap(); // The second child is the variable declaration
        assert_eq!(var_decl.kind(), "variable_declaration");
        
        if let Some(value_node) = var_decl.child_by_field_name("value") {
            // This should be the call expression 'test(5)'
            assert_eq!(value_node.kind(), "call_expression");
            
            // Infer type with scope context
            let typ = infer_type_with_scope(&value_node, source, Some(&analyzer), Some(&tree));
            assert_eq!(typ, Type::Int);
        } else {
            panic!("No value field in variable_declaration");
        }
    }
}
