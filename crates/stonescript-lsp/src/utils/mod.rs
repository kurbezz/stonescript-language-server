//! Utility modules

pub mod scope_analyzer;
pub mod type_inference;

pub use scope_analyzer::{FunctionStub, ScopeAnalyzer, Variable};
pub use type_inference::{infer_type, infer_type_with_scope};
