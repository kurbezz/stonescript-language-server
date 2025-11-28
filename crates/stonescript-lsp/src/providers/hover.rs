//! Hover provider with type information

use crate::data::*;
use crate::utils::{infer_type_with_scope, ScopeAnalyzer, FunctionStub};
use stonescript_parser::ast::{Expression, Position as AstPosition, Program, Statement};
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
        ast: &Program,
        position: Position,
        source: &str,
        scope: &ScopeAnalyzer,
    ) -> Option<Hover> {
        // Convert LSP position to AST position
        let ast_pos = AstPosition::new(position.line as usize, position.character as usize);

        // Try to find expression at position
        if let Some(expr) = self.find_expression_at_position(ast, ast_pos) {
            return self.hover_for_expression(expr, scope);
        }

        // Try to find statement at position
        if let Some(stmt) = self.find_statement_at_position(ast, ast_pos) {
            if let Some(hover) = self.hover_for_statement(stmt, scope) {
                return Some(hover);
            }
        }

        // Fallback to text-based analysis
        self.text_based_hover(position, source, scope)
    }

    fn hover_for_expression(&self, expr: &Expression, scope: &ScopeAnalyzer) -> Option<Hover> {
        match expr {
            Expression::Identifier(name, _) => {
                // Check if it's a game state query
                if let Some(stripped) = name.strip_prefix('?') {
                    return self.hover_for_game_state(stripped);
                }

                // Check if it's a user-defined function
                if let Some(func) = scope.find_function(name) {
                    return self.hover_for_user_function(func);
                }

                // Check if it's a variable
                if let Some(var) = scope.find_variable(name) {
                    let type_info = format!("var {}: {}", name, var.inferred_type);
                    return Some(Hover {
                        contents: HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: format!(
                                "```stonescript\n{}\n```\n\nVariable in scope",
                                type_info
                            ),
                        }),
                        range: None,
                    });
                }

                // Check if it's a built-in function or keyword
                self.hover_for_identifier(name)
            }

            Expression::Property {
                object, property, ..
            } => {
                let obj_type = infer_type_with_scope(object, Some(scope));
                self.hover_for_property(&obj_type, property)
            }

            Expression::FunctionCall { function, .. } => {
                // Show hover for the function being called
                self.hover_for_expression(function, scope)
            }

            Expression::Integer(val, _) => Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("```stonescript\n{}: Int\n```", val),
                }),
                range: None,
            }),

            Expression::Float(val, _) => Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("```stonescript\n{}: Float\n```", val),
                }),
                range: None,
            }),

            Expression::Boolean(val, _) => Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("```stonescript\n{}: Bool\n```", val),
                }),
                range: None,
            }),

            Expression::String(val, _) => Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("```stonescript\n\"{}\": String\n```", val),
                }),
                range: None,
            }),

            _ => {
                // For other expressions, show their inferred type
                let expr_type = infer_type_with_scope(expr, Some(scope));
                if !matches!(expr_type, Type::Unknown) {
                    Some(Hover {
                        contents: HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: format!("```stonescript\nType: {}\n```", expr_type),
                        }),
                        range: None,
                    })
                } else {
                    None
                }
            }
        }
    }

    fn hover_for_statement(&self, stmt: &Statement, _scope: &ScopeAnalyzer) -> Option<Hover> {
        match stmt {
            Statement::Command { name, .. } => {
                // Show hover for command keywords
                self.hover_for_identifier(name)
            }
            _ => None,
        }
    }

    fn hover_for_game_state(&self, name: &str) -> Option<Hover> {
        if let Some(query) = self.game_state.iter().find(|q| q.name == name) {
            let mut content = format!(
                "```stonescript\n?{}: {}\n```\n\n{}",
                query.name, query.return_type, query.description
            );

            // Add properties if available
            if let Some(properties) = query.properties {
                content.push_str("\n\n**Properties:**\n");
                for prop in properties {
                    content.push_str(&format!(
                        "- `{}`: {} - {}\n",
                        prop.name, prop.typ, prop.description
                    ));
                }
            }

            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: content,
                }),
                range: None,
            });
        }
        None
    }

    fn hover_for_property(&self, obj_type: &Type, property: &str) -> Option<Hover> {
        match obj_type {
            Type::Object(obj_name) => {
                // Look up property in game state
                if let Some(query) = get_game_state(obj_name) {
                    if let Some(properties) = query.properties {
                        if let Some(prop) = properties.iter().find(|p| p.name == property) {
                            return Some(Hover {
                                contents: HoverContents::Markup(MarkupContent {
                                    kind: MarkupKind::Markdown,
                                    value: format!(
                                        "```stonescript\n{}.{}: {}\n```\n\n{}",
                                        obj_name, prop.name, prop.typ, prop.description
                                    ),
                                }),
                                range: None,
                            });
                        }
                    }
                }

                // UI properties
                if *obj_name == "Panel" || *obj_name == "UI" {
                    if UI_PROPERTIES.iter().any(|&p| p == property) {
                        return Some(Hover {
                            contents: HoverContents::Markup(MarkupContent {
                                kind: MarkupKind::Markdown,
                                value: format!(
                                    "```stonescript\n{}.{}\n```\n\nUI property",
                                    obj_name, property
                                ),
                            }),
                            range: None,
                        });
                    }

                    // UI methods
                    if UI_METHODS.iter().any(|&m| m == property) {
                        return Some(Hover {
                            contents: HoverContents::Markup(MarkupContent {
                                kind: MarkupKind::Markdown,
                                value: format!(
                                    "```stonescript\n{}.{}()\n```\n\nUI method",
                                    obj_name, property
                                ),
                            }),
                            range: None,
                        });
                    }
                }

                None
            }
            _ => None,
        }
    }

    fn hover_for_identifier(&self, name: &str) -> Option<Hover> {
        // Check keywords
        if let Some(keyword) = self.keywords.iter().find(|k| k.name == name) {
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

        // Check built-in functions (all namespaces)
        if let Some(func) = get_function(name) {
            return self.hover_for_function(func);
        }

        None
    }

    fn hover_for_function(&self, func: &FunctionSignature) -> Option<Hover> {
        let params = func
            .parameters
            .iter()
            .map(|p| {
                if p.optional {
                    format!("{}?: {}", p.name, p.typ)
                } else {
                    format!("{}: {}", p.name, p.typ)
                }
            })
            .collect::<Vec<_>>()
            .join(", ");

        let sig = if func.namespace.is_empty() {
            format!("func {}({}) -> {}", func.name, params, func.return_type)
        } else {
            format!(
                "func {}.{}({}) -> {}",
                func.namespace, func.name, params, func.return_type
            )
        };

        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("```stonescript\n{}\n```\n\n{}", sig, func.description),
            }),
            range: None,
        })
    }

    fn hover_for_user_function(&self, func: &FunctionStub) -> Option<Hover> {
        let params = func.parameters.join(", ");
        let sig = format!("func {}({})", func.name, params);

        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("```stonescript\n{}\n```\n\nUser-defined function", sig),
            }),
            range: None,
        })
    }

    fn text_based_hover(
        &self,
        position: Position,
        source: &str,
        scope: &ScopeAnalyzer,
    ) -> Option<Hover> {
        let line_text = source.lines().nth(position.line as usize)?;
        let word = self.extract_word_at_position(line_text, position.character as usize)?;

        // Check if it's a game state query
        if let Some(stripped) = word.strip_prefix('?') {
            return self.hover_for_game_state(stripped);
        }

        // Check if it's a user-defined function
        if let Some(func) = scope.find_function(&word) {
            return self.hover_for_user_function(func);
        }

        // Check if it's a variable
        if let Some(var) = scope.find_variable(&word) {
            let type_info = format!("var {}: {}", word, var.inferred_type);
            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("```stonescript\n{}\n```\n\nVariable in scope", type_info),
                }),
                range: None,
            });
        }

        // Fallback to identifier lookup
        self.hover_for_identifier(&word)
    }

    fn find_expression_at_position<'a>(
        &self,
        ast: &'a Program,
        pos: AstPosition,
    ) -> Option<&'a Expression> {
        for stmt in &ast.statements {
            if let Some(expr) = self.find_expression_in_statement(stmt, pos) {
                return Some(expr);
            }
        }
        None
    }

    fn find_expression_in_statement<'a>(
        &self,
        stmt: &'a Statement,
        pos: AstPosition,
    ) -> Option<&'a Expression> {
        match stmt {
            Statement::Condition {
                condition,
                then_block,
                else_ifs,
                else_block,
                span,
            } => {
                if span.contains_position(pos) {
                    if condition.span().contains_position(pos) {
                        return self.find_deepest_expression(condition, pos);
                    }
                    for s in then_block {
                        if let Some(expr) = self.find_expression_in_statement(s, pos) {
                            return Some(expr);
                        }
                    }
                    for elif in else_ifs {
                        if elif.span.contains_position(pos) {
                            if elif.condition.span().contains_position(pos) {
                                return self.find_deepest_expression(&elif.condition, pos);
                            }
                            for s in &elif.block {
                                if let Some(expr) = self.find_expression_in_statement(s, pos) {
                                    return Some(expr);
                                }
                            }
                        }
                    }
                    if let Some(else_stmts) = else_block {
                        for s in else_stmts {
                            if let Some(expr) = self.find_expression_in_statement(s, pos) {
                                return Some(expr);
                            }
                        }
                    }
                }
                None
            }
            Statement::Assignment {
                target,
                value,
                span,
                ..
            } => {
                if span.contains_position(pos) {
                    if target.span().contains_position(pos) {
                        return self.find_deepest_expression(target, pos);
                    }
                    if value.span().contains_position(pos) {
                        return self.find_deepest_expression(value, pos);
                    }
                }
                None
            }
            Statement::ExpressionStatement { expression, span } => {
                if span.contains_position(pos) && expression.span().contains_position(pos) {
                    self.find_deepest_expression(expression, pos)
                } else {
                    None
                }
            }
            Statement::Return { value, span } => {
                if span.contains_position(pos) {
                    if let Some(expr) = value {
                        if expr.span().contains_position(pos) {
                            return self.find_deepest_expression(expr, pos);
                        }
                    }
                }
                None
            }
            Statement::Output { text, span, .. } => {
                if span.contains_position(pos) && text.span().contains_position(pos) {
                    self.find_deepest_expression(text, pos)
                } else {
                    None
                }
            }
            Statement::For {
                range, body, span, ..
            } => {
                if span.contains_position(pos) {
                    if range.0.span().contains_position(pos) {
                        return self.find_deepest_expression(&range.0, pos);
                    }
                    if range.1.span().contains_position(pos) {
                        return self.find_deepest_expression(&range.1, pos);
                    }
                    for s in body {
                        if let Some(expr) = self.find_expression_in_statement(s, pos) {
                            return Some(expr);
                        }
                    }
                }
                None
            }
            Statement::Command { args, span, .. } => {
                if span.contains_position(pos) {
                    for arg in args {
                        if arg.span().contains_position(pos) {
                            return self.find_deepest_expression(arg, pos);
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn find_deepest_expression<'a>(
        &self,
        expr: &'a Expression,
        pos: AstPosition,
    ) -> Option<&'a Expression> {
        if !expr.span().contains_position(pos) {
            return None;
        }

        match expr {
            Expression::Property { object, .. } => {
                if let Some(inner) = self.find_deepest_expression(object, pos) {
                    Some(inner)
                } else {
                    Some(expr)
                }
            }
            Expression::FunctionCall { function, args, .. } => {
                if let Some(inner) = self.find_deepest_expression(function, pos) {
                    return Some(inner);
                }
                for arg in args {
                    if let Some(inner) = self.find_deepest_expression(arg, pos) {
                        return Some(inner);
                    }
                }
                Some(expr)
            }
            Expression::BinaryOp { left, right, .. } => {
                if let Some(inner) = self.find_deepest_expression(left, pos) {
                    Some(inner)
                } else if let Some(inner) = self.find_deepest_expression(right, pos) {
                    Some(inner)
                } else {
                    Some(expr)
                }
            }
            Expression::UnaryOp { operand, .. } => {
                if let Some(inner) = self.find_deepest_expression(operand, pos) {
                    Some(inner)
                } else {
                    Some(expr)
                }
            }
            Expression::IndexAccess { object, index, .. } => {
                if let Some(inner) = self.find_deepest_expression(object, pos) {
                    return Some(inner);
                }
                if let Some(inner) = self.find_deepest_expression(index, pos) {
                    Some(inner)
                } else {
                    Some(expr)
                }
            }
            Expression::Array { elements, .. } => {
                for elem in elements {
                    if let Some(inner) = self.find_deepest_expression(elem, pos) {
                        return Some(inner);
                    }
                }
                Some(expr)
            }
            _ => Some(expr),
        }
    }

    fn find_statement_at_position<'a>(
        &self,
        ast: &'a Program,
        pos: AstPosition,
    ) -> Option<&'a Statement> {
        for stmt in &ast.statements {
            if let Some(span) = stmt.span() {
                if span.contains_position(pos) {
                    return Some(stmt);
                }
            }
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

        // Find word boundaries (including ? for game state queries)
        let mut start = col;
        while start > 0
            && (chars[start - 1].is_alphanumeric()
                || chars[start - 1] == '_'
                || chars[start - 1] == '?')
        {
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
        assert_eq!(
            provider.extract_word_at_position("?loc", 1),
            Some("?loc".to_string())
        );
    }
}
