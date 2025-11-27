//! Definition provider (go-to-definition)

use tower_lsp::lsp_types::*;
use crate::utils::ScopeAnalyzer;
use stonescript_parser::Program;

pub struct DefinitionProvider;

impl DefinitionProvider {
    pub fn new() -> Self {
        Self
    }

    pub fn provide_definition(
        &self,
        _ast: &Program,
        position: Position,
        source: &str,
        scope: &ScopeAnalyzer,
        _uri: &Url,
    ) -> Option<GotoDefinitionResponse> {
        // Simplified approach: extract word at cursor
        let line = source.lines().nth(position.line as usize)?;
        let word = self.extract_word_at_position(line, position.character as usize)?;
        
        // Check if variable exists in scope
        if scope.has_variable(&word) {
            // TODO: Once AST tracks source positions, return actual definition location
            // For now, returning None since we can't determine position without tree-sitter
            None
        } else {
            None
        }
    }

    fn extract_word_at_position(&self, line: &str, col: usize) -> Option<String> {
        if col > line.len() {
            return None;
        }

        let chars: Vec<char> = line.chars().collect();
        if chars.is_empty() {
            return None;
        }

        let col = col.min(chars.len());

        // Find word boundaries
        let mut start = col;
        while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
            start -= 1;
        }

        let mut end = col;
        while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
            end += 1;
        }

        if start < end {
            Some(chars[start..end].iter().collect())
        } else {
            None
        }
    }
}
