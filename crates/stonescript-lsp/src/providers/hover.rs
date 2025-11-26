//! Hover provider

use crate::data::native_functions::get_function;
use crate::data::*;
use crate::utils::{type_inference, ScopeAnalyzer};
use tower_lsp::lsp_types::*;
use tree_sitter::{Point, Tree};

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

        let node = tree
            .root_node()
            .named_descendant_for_point_range(point, point)?;

        match node.kind() {
            "identifier" => {
                let text = node.utf8_text(source.as_bytes()).ok()?;

                // Check if this identifier is part of a call expression
                if let Some(parent) = node.parent() {
                    if parent.kind() == "call_expression" {
                        // This is a function call - show function info
                        if let Some(func_node) = parent.child_by_field_name("function") {
                            if func_node.id() == node.id() {
                                // Hovering directly over the function name in a call
                                return self.hover_for_identifier(text, &node, source, scope, tree);
                            }
                        }
                    } else if parent.kind() == "member_expression" {
                        // Check if we are the property part of the member expression
                        if let Some(prop) = parent.child_by_field_name("property") {
                            if prop.id() == node.id() {
                                return self.hover_for_member_expression(&parent, source);
                            }
                        }
                    }
                }

                self.hover_for_identifier(text, &node, source, scope, tree)
            }
            "member_expression" => self.hover_for_member_expression(&node, source),
            _ => {
                // Try to get text and see if it's a keyword
                let text = node.utf8_text(source.as_bytes()).ok()?;
                self.hover_for_keyword(text)
            }
        }
    }

    fn hover_for_identifier(
        &self,
        text: &str,
        node: &tree_sitter::Node,
        source: &str,
        scope: &ScopeAnalyzer,
        tree: &Tree,
    ) -> Option<Hover> {
        // Check if it's a keyword
        if let Some(hover) = self.hover_for_keyword(text) {
            return Some(hover);
        }

        // Check if it's a game state query
        if let Some(query) = self.game_state.iter().find(|q| q.name == text) {
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

        // Check if it's a user function
        if let Some(func) = scope.get_function(text) {
            let mut func_with_return = func.clone();

            // Try to infer return type if not already set and we have body location
            if func_with_return.return_type.is_none() {
                if let (Some(body_start), Some(body_end)) =
                    (func.body_start_byte, func.body_end_byte)
                {
                    func_with_return.return_type = type_inference::infer_function_return_type(
                        tree, body_start, body_end, source, Some(scope),
                    );
                }
            }

            // Build function signature
            let params = func_with_return.parameters
                .iter()
                .map(|p| format!("{}", p))
                .collect::<Vec<_>>()
                .join(", ");
            let return_type_str = if let Some(ret_type) = &func_with_return.return_type {
                format!(" -> {}", ret_type)
            } else {
                String::new()
            };

            let mut hover_text = format!(
                "```stonescript\nfunc {}({}){}\n```",
                func_with_return.name, params, return_type_str
            );

            // Add documentation if available
            if let Some(doc) = &func_with_return.doc_comment {
                hover_text.push_str("\n\n");
                hover_text.push_str(doc);
            }

            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: hover_text,
                }),
                range: None,
            });
        }

        // Check if it's a variable and infer its type
        let variables = scope.find_variables_at(node.start_byte());
        if let Some(var) = variables.iter().find(|v| v.name == text) {
            // Find the variable declaration node using byte range
            let var_decl_node = tree
                .root_node()
                .named_descendant_for_byte_range(var.start_byte, var.end_byte);

            let inferred_type = if let Some(decl) = var_decl_node {
                // Get the value field from the declaration
                if let Some(value_node) = decl.child_by_field_name("value") {
                    type_inference::infer_type_with_scope(&value_node, source, Some(scope), Some(tree))
                } else {
                    Type::Unknown
                }
            } else {
                Type::Unknown
            };

            let type_str = if inferred_type != Type::Unknown {
                format!("{}", inferred_type)
            } else {
                "unknown".to_string()
            };

            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("```stonescript\nvar {}: {}\n```", text, type_str),
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
                            value: format!(
                                "```stonescript\n{}.{}: {}\n```\n\n{}",
                                object, property, prop.typ, prop.description
                            ),
                        }),
                        range: None,
                    });
                }
            }
        }

        // Check for namespace functions
        if let Some(func) = get_function(object, property) {
            let params = func
                .parameters
                .iter()
                .map(|p| format!("{}: {}", p.name, p.typ))
                .collect::<Vec<_>>()
                .join(", ");

            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!(
                        "```stonescript\n{}.{}({}) -> {}\n```\n\n{}",
                        func.namespace, func.name, params, func.return_type, func.description
                    ),
                }),
                range: None,
            });
        }

        None
    }

    fn hover_for_keyword(&self, text: &str) -> Option<Hover> {
        let keyword = self.keywords.iter().find(|k| k.name == text)?;

        let examples = if !keyword.examples.is_empty() {
            format!(
                "\n\n**Examples:**\n```stonescript\n{}\n```",
                keyword.examples.join("\n")
            )
        } else {
            String::new()
        };

        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!(
                    "```stonescript\n{}\n```\n\n{}\n\n**Usage:** `{}`{}",
                    keyword.name, keyword.description, keyword.usage, examples
                ),
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
