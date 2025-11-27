//! Type inference from AST

use crate::data::{
    get_function, get_game_state, Type, MATH_FUNCTIONS, MUSIC_FUNCTIONS, STORAGE_FUNCTIONS,
    STRING_FUNCTIONS, UI_FUNCTIONS,
};
use crate::utils::ScopeAnalyzer;
use stonescript_parser::ast::{BinaryOperator, Expression, Statement, UnaryOperator};

/// Infer type from an AST expression
pub fn infer_type(expression: &Expression) -> Type {
    infer_type_with_scope(expression, None)
}

/// Infer type from an AST expression with scope information
pub fn infer_type_with_scope(expression: &Expression, scope: Option<&ScopeAnalyzer>) -> Type {
    match expression {
        // Literals have known types
        Expression::Integer(_, _) => Type::Int,
        Expression::Float(_, _) => Type::Float,
        Expression::Boolean(_, _) => Type::Bool,
        Expression::String(_, _) => Type::String,

        // String interpolation always produces a string
        Expression::Interpolation(_, _) => Type::String,

        // Arrays - try to infer element type
        Expression::Array { elements, .. } => {
            if elements.is_empty() {
                Type::Array(&Type::Unknown)
            } else {
                // Infer type from first element
                let elem_type = infer_type_with_scope(&elements[0], scope);
                Type::Array(Box::leak(Box::new(elem_type)))
            }
        }

        // Identifier - look up in scope or check game state
        Expression::Identifier(name, _) => {
            // Check if it's a game state query (starts with ?)
            if let Some(stripped) = name.strip_prefix('?') {
                if let Some(query) = get_game_state(stripped) {
                    return query.return_type.clone();
                }
            }

            // Check scope for variable type
            if let Some(scope) = scope {
                if let Some(var) = scope.find_variable(name) {
                    return var.inferred_type.clone();
                }
            }

            Type::Unknown
        }

        // Property access
        Expression::Property {
            object, property, ..
        } => {
            let obj_type = infer_type_with_scope(object, scope);
            infer_property_type(&obj_type, property)
        }

        // Index access - return element type of array
        Expression::IndexAccess { object, .. } => {
            let obj_type = infer_type_with_scope(object, scope);
            match obj_type {
                Type::Array(elem_type) => (*elem_type).clone(),
                Type::String => Type::String, // String indexing returns string
                _ => Type::Unknown,
            }
        }

        // Function call - look up function signature
        Expression::FunctionCall { function, args, .. } => {
            infer_function_return_type(function, args, scope)
        }

        // Binary operations
        Expression::BinaryOp {
            left, op, right, ..
        } => infer_binary_op_type(left, *op, right, scope),

        // Unary operations
        Expression::UnaryOp { op, operand, .. } => infer_unary_op_type(*op, operand, scope),

        // New expressions create objects
        Expression::New { path, .. } => {
            // Extract object type from path (last segment)
            let obj_name = path.split('/').last().unwrap_or(path);
            Type::Object(Box::leak(obj_name.to_string().into_boxed_str()))
        }
    }
}

/// Infer type of a property access
fn infer_property_type(object_type: &Type, property: &str) -> Type {
    match object_type {
        Type::Object(obj_name) => {
            // Look up property in game state queries
            if let Some(query) = get_game_state(obj_name) {
                if let Some(properties) = query.properties {
                    if let Some(prop) = properties.iter().find(|p| p.name == property) {
                        return prop.typ.clone();
                    }
                }
            }

            // Special handling for known object types
            match *obj_name {
                "Location" => match property {
                    "id" | "name" => Type::String,
                    "stars" | "gp" => Type::Int,
                    "begin" | "loop" => Type::Bool,
                    _ => Type::Unknown,
                },
                "Foe" => match property {
                    "id" | "name" => Type::String,
                    "hp" | "maxhp" | "armor" | "distance" | "damage" | "count" => Type::Int,
                    _ => Type::Unknown,
                },
                "Screen" => match property {
                    "w" | "h" => Type::Int,
                    _ => Type::Unknown,
                },
                "Input" => match property {
                    "x" | "y" => Type::Int,
                    _ => Type::Unknown,
                },
                "Cooldown" => Type::Float, // All cooldown properties are floats
                "UI" | "Panel" => match property {
                    "root" => Type::Object("Panel"),
                    "x" | "y" | "w" | "h" | "minW" | "minH" | "maxW" | "maxH" => Type::Int,
                    "visible" | "enabled" => Type::Bool,
                    "text" | "id" | "color" => Type::String,
                    "alpha" => Type::Float,
                    _ => Type::Unknown,
                },
                _ => Type::Unknown,
            }
        }
        _ => Type::Unknown,
    }
}

/// Infer return type of a function call
fn infer_function_return_type(
    function: &Expression,
    _args: &[Expression],
    scope: Option<&ScopeAnalyzer>,
) -> Type {
    match function {
        // Method call: obj.method(args)
        Expression::Property {
            object, property, ..
        } => {
            let obj_type = infer_type_with_scope(object, scope);

            // Check for method on specific object types
            match obj_type {
                Type::Object(obj_name) => match obj_name {
                    "Foe" => match property.as_str() {
                        "GetCount" => Type::Int,
                        _ => Type::Unknown,
                    },
                    "Panel" | "UI" => match property.as_str() {
                        "Add" | "AddButton" | "AddLabel" | "AddPanel" | "AddBar" => {
                            Type::Object("Panel")
                        }
                        "Remove" | "Clear" | "SetText" | "SetColor" | "SetVisible"
                        | "SetEnabled" | "SetPos" | "SetSize" | "SetAlpha" => Type::Unknown,
                        "GetChild" => Type::Object("Panel"),
                        _ => Type::Unknown,
                    },
                    _ => Type::Unknown,
                },
                Type::String => {
                    // String methods
                    match property.as_str() {
                        "Length" => Type::Int,
                        "ToUpper" | "ToLower" | "Trim" | "Substring" | "Replace" => Type::String,
                        "Contains" | "StartsWith" | "EndsWith" => Type::Bool,
                        "Split" => Type::Array(&Type::String),
                        "IndexOf" | "LastIndexOf" => Type::Int,
                        _ => Type::Unknown,
                    }
                }
                Type::Array(_) => {
                    // Array methods
                    match property.as_str() {
                        "Length" | "Count" | "IndexOf" | "LastIndexOf" => Type::Int,
                        "Add" | "Remove" | "RemoveAt" | "Clear" | "Insert" => Type::Unknown,
                        "Contains" => Type::Bool,
                        "Get" => {
                            // Return element type
                            if let Type::Array(elem_type) = obj_type {
                                (*elem_type).clone()
                            } else {
                                Type::Unknown
                            }
                        }
                        _ => Type::Unknown,
                    }
                }
                _ => Type::Unknown,
            }
        }

        // Direct function call or namespace function
        Expression::Identifier(name, _) => {
            // Check built-in functions (without namespace)
            if let Some(func) = get_function(name) {
                return func.return_type.clone();
            }

            Type::Unknown
        }

        // Namespace function call (treated as property access)
        _ => Type::Unknown,
    }
}

/// Infer type of binary operation
fn infer_binary_op_type(
    left: &Expression,
    op: BinaryOperator,
    right: &Expression,
    scope: Option<&ScopeAnalyzer>,
) -> Type {
    match op {
        // Comparison operators always return bool
        BinaryOperator::Equal
        | BinaryOperator::NotEqual
        | BinaryOperator::Less
        | BinaryOperator::LessEqual
        | BinaryOperator::Greater
        | BinaryOperator::GreaterEqual => Type::Bool,

        // Logical operators return bool
        BinaryOperator::And | BinaryOperator::Or => Type::Bool,

        // Arithmetic operators
        BinaryOperator::Add => {
            let left_type = infer_type_with_scope(left, scope);
            let right_type = infer_type_with_scope(right, scope);

            // String concatenation
            if matches!(left_type, Type::String) || matches!(right_type, Type::String) {
                return Type::String;
            }

            // Float if either operand is float
            if matches!(left_type, Type::Float) || matches!(right_type, Type::Float) {
                return Type::Float;
            }

            // Otherwise int (or unknown)
            if matches!(left_type, Type::Int) && matches!(right_type, Type::Int) {
                Type::Int
            } else {
                Type::Unknown
            }
        }

        BinaryOperator::Subtract | BinaryOperator::Multiply | BinaryOperator::Modulo => {
            let left_type = infer_type_with_scope(left, scope);
            let right_type = infer_type_with_scope(right, scope);

            // Float if either operand is float
            if matches!(left_type, Type::Float) || matches!(right_type, Type::Float) {
                Type::Float
            } else if matches!(left_type, Type::Int) && matches!(right_type, Type::Int) {
                Type::Int
            } else {
                Type::Unknown
            }
        }

        BinaryOperator::Divide => {
            // Division always returns float in most languages
            Type::Float
        }
    }
}

/// Infer type of unary operation
fn infer_unary_op_type(
    op: UnaryOperator,
    operand: &Expression,
    scope: Option<&ScopeAnalyzer>,
) -> Type {
    match op {
        UnaryOperator::Not => Type::Bool,
        UnaryOperator::Negate => {
            let operand_type = infer_type_with_scope(operand, scope);
            // Preserve numeric type
            match operand_type {
                Type::Int => Type::Int,
                Type::Float => Type::Float,
                _ => Type::Unknown,
            }
        }
        UnaryOperator::Increment | UnaryOperator::Decrement => {
            let operand_type = infer_type_with_scope(operand, scope);
            // Preserve numeric type
            match operand_type {
                Type::Int => Type::Int,
                Type::Float => Type::Float,
                _ => Type::Unknown,
            }
        }
    }
}

/// Infer type from a statement (for return statements, assignments, etc.)
pub fn infer_type_from_statement(statement: &Statement, scope: Option<&ScopeAnalyzer>) -> Type {
    match statement {
        Statement::Return { value, .. } => {
            if let Some(expr) = value {
                infer_type_with_scope(expr, scope)
            } else {
                Type::Unknown
            }
        }
        Statement::Assignment { value, .. } => infer_type_with_scope(value, scope),
        Statement::ExpressionStatement { expression, .. } => {
            infer_type_with_scope(expression, scope)
        }
        _ => Type::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stonescript_parser::ast::{Position, Span};

    fn dummy_span() -> Span {
        Span::new(Position::new(0, 0), Position::new(0, 0))
    }

    #[test]
    fn test_literal_types() {
        assert_eq!(
            infer_type(&Expression::Integer(42, dummy_span())),
            Type::Int
        );
        assert_eq!(
            infer_type(&Expression::Float(3.14, dummy_span())),
            Type::Float
        );
        assert_eq!(
            infer_type(&Expression::Boolean(true, dummy_span())),
            Type::Bool
        );
        assert_eq!(
            infer_type(&Expression::String("hello".to_string(), dummy_span())),
            Type::String
        );
    }

    #[test]
    fn test_binary_op_types() {
        let left = Expression::Integer(1, dummy_span());
        let right = Expression::Integer(2, dummy_span());

        // Arithmetic
        let add_type = infer_binary_op_type(&left, BinaryOperator::Add, &right, None);
        assert_eq!(add_type, Type::Int);

        // Comparison
        let cmp_type = infer_binary_op_type(&left, BinaryOperator::Less, &right, None);
        assert_eq!(cmp_type, Type::Bool);
    }

    #[test]
    fn test_unary_op_types() {
        let operand = Expression::Integer(42, dummy_span());

        assert_eq!(
            infer_unary_op_type(
                UnaryOperator::Not,
                &Expression::Boolean(true, dummy_span()),
                None
            ),
            Type::Bool
        );
        assert_eq!(
            infer_unary_op_type(UnaryOperator::Negate, &operand, None),
            Type::Int
        );
    }

    #[test]
    fn test_string_concatenation() {
        let str_expr = Expression::String("hello".to_string(), dummy_span());
        let int_expr = Expression::Integer(42, dummy_span());

        let result = infer_binary_op_type(&str_expr, BinaryOperator::Add, &int_expr, None);
        assert_eq!(result, Type::String);
    }
}
