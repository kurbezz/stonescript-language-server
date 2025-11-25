//! Formatting provider

use tower_lsp::lsp_types::*;

pub struct FormattingProvider;

impl FormattingProvider {
    pub fn new() -> Self {
        Self
    }

    pub fn provide_formatting(&self, _source: &str) -> Vec<TextEdit> {
        // Basic formatting: ensure consistent indentation
        // For now, return empty - full formatting would need more work
        vec![]
    }
}
