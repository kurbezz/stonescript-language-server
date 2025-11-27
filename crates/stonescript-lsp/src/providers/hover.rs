//! Hover provider

use crate::data::*;
use crate::utils::ScopeAnalyzer;
use stonescript_parser::Program;
use tower_lsp::lsp_types::*;

pub struct HoverProvider {
    keywords: &'static [KeywordInfo],
    game_state: &'static [GameStateQuery],
}

impl HoverProvider {
    pub fn new() -> Self {
        Self {
            keywords: KEYWORDS,
            game_state: GAME_STATE_QUERIES,
        }
    }

    pub fn provide_hover(
        &self,
        _ast: &Program,
        position: Position,
        source: &str,
        scope: &ScopeAnalyzer,
    ) -> Option<Hover> {
        // TODO: Implement full AST-based hover
        // For now, use text-based fallback for basic functionality

        let line_text = source.lines().nth(position.line as usize)?;
        let word = self.extract_word_at_position(line_text, position.character as usize)?;

        // Try to provide hover for keywords
        if let Some(keyword) = self.keywords.iter().find(|k| k.name == word) {
            let examples = if !keyword.examples.is_empty() {
                format!(
                    "\n\n**Examples:**\n```stonescript\n{}\n```",
                    keyword.examples.join("\n")
                )
            } else {
                String::new()
            };

            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!(
                        "```stonescript\n{}\n```\n\n{}\n\n**Usage:** `{}`{}",
                        keyword.name, keyword.description, keyword.usage, examples
                    ),
                }),
                range: None,
            });
        }

        // Try to provide hover for game state queries
        if let Some(query) = self.game_state.iter().find(|q| q.name == word) {
            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!(
                        "```stonescript\n{}: {}\n```\n\n{}",
                        query.name, query.return_type, query.description
                    ),
                }),
                range: None,
            });
        }

        // Try to provide hover for variables in scope
        if let Some(_var) = scope.find_variable(&word) {
            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("```stonescript\nvar {}\n```\n\nVariable in scope", word),
                }),
                range: None,
            });
        }

        None
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

impl Default for HoverProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hover_provider_creation() {
        let provider = HoverProvider::new();
        assert!(!provider.keywords.is_empty());
    }

    #[test]
    fn test_extract_word() {
        let provider = HoverProvider::new();
        assert_eq!(
            provider.extract_word_at_position("hello world", 3),
            Some("hello".to_string())
        );
        assert_eq!(
            provider.extract_word_at_position("hello world", 7),
            Some("world".to_string())
        );
    }
}
