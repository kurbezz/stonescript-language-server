//! Diagnostics provider

use crate::data::{native_functions, Type};
use crate::utils::ScopeAnalyzer;
use tower_lsp::lsp_types::*;
use stonescript_parser::{Program, Visitor, Statement, Expression};

pub struct DiagnosticsProvider;

impl DiagnosticsProvider {
    pub fn new() -> Self {
        Self
    }

    pub fn provide_diagnostics(
        &self,
        _ast: &Program,
        _source: &str,
        _scope: &ScopeAnalyzer,
    ) -> Vec<Diagnostic> {
        // Simplified diagnostics - parse errors are already handled by server.rs
        // TODO: Implement AST-based semantic analysis:
        // - Walk AST to find undefined variable references
        // - Check function call types (would need updated type_inference)
        // For now, relying on parse error reporting only
        
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostics_provider() {
        let provider = DiagnosticsProvider::new();
        let _ = provider;
    }
}
