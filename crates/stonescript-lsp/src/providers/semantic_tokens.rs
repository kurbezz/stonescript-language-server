//! Semantic tokens provider

use tower_lsp::lsp_types::*;
use stonescript_parser::Program;

pub struct SemanticTokensProvider {
    token_types: Vec<SemanticTokenType>,
}

impl SemanticTokensProvider {
    pub fn new() -> Self {
        Self {
            token_types: vec![
                SemanticTokenType::VARIABLE,
                SemanticTokenType::FUNCTION,
                SemanticTokenType::KEYWORD,
                SemanticTokenType::OPERATOR,
                SemanticTokenType::NUMBER,
                SemanticTokenType::STRING,
            ],
        }
    }

    pub fn legend(&self) -> SemanticTokensLegend {
        SemanticTokensLegend {
            token_types: self.token_types.clone(),
            token_modifiers: vec![],
        }
    }

    pub fn provide_semantic_tokens(&self, _ast: &Program, _source: &str) -> SemanticTokens {
        // TODO: Implement AST-based semantic tokens using Visitor pattern
        // For now, return empty tokens since we'd need position information from AST
        // This can be enhanced once AST nodes track their source positions
        
        SemanticTokens {
            result_id: None,
            data: vec![],
        }
    }
}
