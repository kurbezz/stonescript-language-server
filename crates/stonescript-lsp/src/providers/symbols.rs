//! Symbols provider (document outline)

use tower_lsp::lsp_types::*;
use crate::utils::ScopeAnalyzer;
use stonescript_parser::Program;

pub struct SymbolsProvider;

impl SymbolsProvider {
    pub fn new() -> Self {
        Self
    }

    pub fn provide_symbols(
        &self,
        _ast: &Program,
        scope: &ScopeAnalyzer,
        _source: &str,
    ) -> Vec<DocumentSymbol> {
        let mut symbols = Vec::new();

        // Add variables from scope
        // Note: Without source position tracking in AST, we use placeholder ranges
        for var in scope.get_all_variables() {
            #[allow(deprecated)]
            symbols.push(DocumentSymbol {
                name: var.name.clone(),
                detail: Some("variable".to_string()),
                kind: SymbolKind::VARIABLE,
                range: Range {
                    start: Position { line: 0, character: 0 },
                    end: Position { line: 0, character: 0 },
                },
                selection_range: Range {
                    start: Position { line: 0, character: 0 },
                    end: Position { line: 0, character: 0 },
                },
                children: None,
                tags: None,
                deprecated: None,
            });
        }

        // Note: StoneScript doesn't have user-defined functions
        // Only variables are shown in the symbol outline

        symbols
    }
}
