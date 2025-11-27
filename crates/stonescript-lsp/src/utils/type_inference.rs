//! Type inference from AST

use stonescript_parser::ast::Expression;
use crate::data::Type;
use crate::utils::ScopeAnalyzer;

/// Infer type from an AST expression
pub fn infer_type(_expression: &Expression) -> Type {
    // TODO: Implement AST-based type inference
    // For now, return Unknown to avoid compilation errors
    // This can be implemented later when needed for diagnostics
    Type::Unknown
}

/// Infer type from an AST expression with scope information
pub fn infer_type_with_scope(_expression: &Expression, _scope: Option<&ScopeAnalyzer>) -> Type {
    // TODO: Implement AST-based type inference with scope
    Type::Unknown
}
