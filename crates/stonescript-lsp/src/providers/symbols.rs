//! Symbols provider (document outline)

use tower_lsp::lsp_types::*;
use tree_sitter::Tree;
use crate::utils::ScopeAnalyzer;

pub struct SymbolsProvider;

impl SymbolsProvider {
    pub fn new() -> Self {
        Self
    }

    pub fn provide_symbols(
        &self,
        _tree: &Tree,
        scope: &ScopeAnalyzer,
        source: &str,
    ) -> Vec<DocumentSymbol> {
        let mut symbols = Vec::new();

        // Add functions
        for func in scope.get_functions() {
            #[allow(deprecated)]
            symbols.push(DocumentSymbol {
                name: func.name.clone(),
                detail: Some(format!("({})", func.parameters.join(", "))),
                kind: SymbolKind::FUNCTION,
                range: self.byte_range_to_lsp(func.start_byte, func.end_byte, source),
                selection_range: self.byte_range_to_lsp(func.start_byte, func.end_byte, source),
                children: None,
                tags: None,
                deprecated: None,
            });
        }

        // Variables are in scopes, would need to collect them
        // For now, just functions

        symbols
    }

    fn byte_range_to_lsp(&self, start: usize, end: usize, source: &str) -> Range {
        let start_pos = self.byte_to_position(start, source);
        let end_pos = self.byte_to_position(end, source);
        Range { start: start_pos, end: end_pos }
    }

    fn byte_to_position(&self, byte: usize, source: &str) -> Position {
        let mut line = 0;
        let mut col = 0;
        
        for (i, ch) in source.chars().enumerate() {
            if i >= byte {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
        }

        Position { line: line as u32, character: col as u32 }
    }
}
