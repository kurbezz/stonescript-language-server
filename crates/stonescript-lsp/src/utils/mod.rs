//! Utility modules

pub mod scope_analyzer;
pub mod type_inference;

pub use scope_analyzer::{ScopeAnalyzer, Variable, Function};
pub use type_inference::infer_type;
