//! Hover provider

use tower_lsp::lsp_types::*;
use tree_sitter::{Tree, Point};
use crate::data::*;
use crate::data::native_functions::{MATH_FUNCTIONS, STRING_FUNCTIONS, STORAGE_FUNCTIONS, get_function};
use crate::utils::ScopeAnalyzer;

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
        tree: &Tree,
        position: Position,
        source: &str,
        scope: &ScopeAnalyzer,
    ) -> Option<Hover> {
        let point = Point::new(position.line as usize, position.character as usize);
        
        let node = tree.root_node()
            .named_descendant_for_point_range(point, point)?;
        
        match node.kind() {
            "identifier" => {
                let text = node.utf8_text(source.as_bytes()).ok()?;
                self.hover_for_identifier(text, scope)
            }
            "member_expression" => {
                self.hover_for_member_expression(&node, source)
            }
            _ => {
                // Try to get text and see if it's a keyword
                let text = node.utf8_text(source.as_bytes()).ok()?;
                self.hover_for_keyword(text)
            }
        }
    }

    fn hover_for_identifier(&self, text: &str, scope: &ScopeAnalyzer) -> Option<Hover> {
        // Check if it's a keyword
        if let Some(hover) = self.hover_for_keyword(text) {
            return Some(hover);
        }

        // Check if it's a game state query
        if let Some(query) = self.game_state.iter().find(|q| q.name == text) {
            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("**{}**\n\n{}\n\nType: `{:?}`", 
                        query.name, query.description, query.return_type),
                }),
                range: None,
            });
        }

        // Check if it's a user function
        if let Some(func) = scope.get_function(text) {
            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("```stonescript\nfunc {}({})\n```", 
                        func.name, func.parameters.join(", ")),
                }),
                range: None,
            });
        }

        // Check if it's a variable
        let variables = scope.find_variables_at(0);
        if variables.iter().any(|v| v.name == text) {
            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("**{}**\n\nUser variable", text),
                }),
                range: None,
            });
        }

        None
    }

    fn hover_for_member_expression(&self, node: &tree_sitter::Node, source: &str) -> Option<Hover> {
        let object_node = node.child_by_field_name("object")?;
        let property_node = node.child_by_field_name("property")?;
        
        let object = object_node.utf8_text(source.as_bytes()).ok()?;
        let property = property_node.utf8_text(source.as_bytes()).ok()?;

        // Check for game state properties
        if let Some(query) = self.game_state.iter().find(|q| q.name == object) {
            if let Some(properties) = query.properties {
                if let Some(prop) = properties.iter().find(|p| p.name == property) {
                    return Some(Hover {
                        contents: HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: format!("**{}.{}**\n\n{}\n\nType: `{:?}`",
                                object, property, prop.description, prop.typ),
                        }),
                        range: None,
                    });
                }
            }
        }

        // Check for namespace functions
        if let Some(func) = get_function(object, property) {
            let params = func.parameters.iter()
                .map(|p| format!("{}: {:?}", p.name, p.typ))
                .collect::<Vec<_>>()
                .join(", ");
            
            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("```stonescript\n{}.{}({})\n```\n\n{}\n\nReturns: `{:?}`",
                        func.namespace, func.name, params, func.description, func.return_type),
                }),
                range: None,
            });
        }

        None
    }

    fn hover_for_keyword(&self, text: &str) -> Option<Hover> {
        let keyword = self.keywords.iter().find(|k| k.name == text)?;
        
        let examples = if !keyword.examples.is_empty() {
            format!("\n\n**Examples:**\n```stonescript\n{}\n```", 
                keyword.examples.join("\n"))
        } else {
            String::new()
        };

        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("**{}**\n\n{}\n\n**Usage:** `{}`{}", 
                    keyword.name, keyword.description, keyword.usage, examples),
            }),
            range: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hover_provider() {
        let provider = HoverProvider::new();
        assert!(provider.keywords.len() > 0);
    }
}
