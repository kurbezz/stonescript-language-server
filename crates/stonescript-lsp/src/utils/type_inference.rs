//! Type inference from tree-sitter AST

use tree_sitter::Node;
use crate::data::Type;

/// Infer type from an AST node
pub fn infer_type(node: &Node, source: &str) -> Type {
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
                match object_text {
                    "loc" => {
                        if let Some(prop_node) = node.child_by_field_name("property") {
                            let prop = prop_node.utf8_text(source.as_bytes()).unwrap_or("");
                            return infer_game_state_property("loc", prop);
                        }
                    }
                    "foe" => {
                        if let Some(prop_node) = node.child_by_field_name("property") {
                            let prop = prop_node.utf8_text(source.as_bytes()).unwrap_or("");
                            return infer_game_state_property("foe", prop);
                        }
                    }
                    "math" => {
                        // Math functions return numbers
                        return Type::Float;
                    }
                    "string" => {
                        // String functions return strings or numbers
                        return Type::String;
                    }
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
                    return infer_type(&func_node, source);
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
                    "+" | "-" | "*" | "/" | "%" => Type::Int,
                    "=" | "!" | "<" | ">" | "<=" | ">=" | "&" | "|" => Type::Bool,
                    _ => Type::Unknown,
                }
            } else {
                Type::Unknown
            }
        }
        
        "identifier" => {
            // Check if it's a known game state query
            let text = node.utf8_text(source.as_bytes()).unwrap_or("");
            match text {
                "hp" | "maxhp" | "armor" => Type::Int,
                "time" | "totaltime" => Type::Float,
                "buffs" | "debuffs" => Type::String,
                "loc" => Type::Object("Location"),
                "foe" => Type::Object("Foe"),
                _ => Type::Unknown,
            }
        }
        
        _ => Type::Unknown,
    }
}

fn infer_game_state_property(object: &str, property: &str) -> Type {
    match (object, property) {
        ("loc", "id") | ("loc", "name") => Type::String,
        ("loc", "stars") | ("loc", "gp") => Type::Int,
        ("loc", "begin") | ("loc", "loop") => Type::Bool,
        
        ("foe", "id") | ("foe", "name") => Type::String,
        ("foe", "hp") | ("foe", "maxhp") | ("foe", "armor") => Type::Int,
        ("foe", "distance") | ("foe", "damage") | ("foe", "count") => Type::Int,
        
        _ => Type::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal_types() {
        let source = "10";
        let tree = stonescript_parser::parse(source).unwrap();
        let node = tree.root_node().child(0).unwrap().child(0).unwrap();
        
        let typ = infer_type(&node, source);
        assert_eq!(typ, Type::Int);
    }

    #[test]
    fn test_game_state_type() {
        let source = "?hp";
        let tree = stonescript_parser::parse(source).unwrap();
        // This is just a basic test structure
        // Real implementation would need proper AST navigation
    }
}
