//! Definition provider (go-to-definition)

use crate::utils::ScopeAnalyzer;
use stonescript_parser::ast::{Expression, Position as AstPosition, Program, Statement};
use tower_lsp::lsp_types::*;

pub struct DefinitionProvider;

impl DefinitionProvider {
    pub fn new() -> Self {
        Self
    }

    pub fn provide_definition(
        &self,
        ast: &Program,
        position: Position,
        source: &str,
        _scope: &ScopeAnalyzer,
        uri: &Url,
    ) -> Option<GotoDefinitionResponse> {
        // Extract word at cursor
        let line = source.lines().nth(position.line as usize)?;
        let word = self.extract_word_at_position(line, position.character as usize)?;

        // Convert LSP position to AST position
        let ast_pos = AstPosition::new(position.line as usize, position.character as usize);

        // Check if we're clicking on a function call or identifier
        if let Some(expr) = self.find_expression_at_position(ast, ast_pos) {
            if let Expression::Identifier(name, _) = expr {
                // Look for function definition
                if let Some(def_span) = self.find_function_definition(ast, name) {
                    let location = Location {
                        uri: uri.clone(),
                        range: Range {
                            start: Position {
                                line: def_span.start.line as u32,
                                character: def_span.start.column as u32,
                            },
                            end: Position {
                                line: def_span.end.line as u32,
                                character: def_span.end.column as u32,
                            },
                        },
                    };
                    return Some(GotoDefinitionResponse::Scalar(location));
                }

                // Look for variable assignment (scope-aware)
                if let Some(def_span) = self.find_variable_definition_scoped(ast, name, ast_pos) {
                    let location = Location {
                        uri: uri.clone(),
                        range: Range {
                            start: Position {
                                line: def_span.start.line as u32,
                                character: def_span.start.column as u32,
                            },
                            end: Position {
                                line: def_span.end.line as u32,
                                character: def_span.end.column as u32,
                            },
                        },
                    };
                    return Some(GotoDefinitionResponse::Scalar(location));
                }
            }
        }

        // Fallback: search by word
        if let Some(def_span) = self.find_function_definition(ast, &word) {
            let location = Location {
                uri: uri.clone(),
                range: Range {
                    start: Position {
                        line: def_span.start.line as u32,
                        character: def_span.start.column as u32,
                    },
                    end: Position {
                        line: def_span.end.line as u32,
                        character: def_span.end.column as u32,
                    },
                },
            };
            return Some(GotoDefinitionResponse::Scalar(location));
        }

        if let Some(def_span) = self.find_variable_definition_scoped(ast, &word, ast_pos) {
            let location = Location {
                uri: uri.clone(),
                range: Range {
                    start: Position {
                        line: def_span.start.line as u32,
                        character: def_span.start.column as u32,
                    },
                    end: Position {
                        line: def_span.end.line as u32,
                        character: def_span.end.column as u32,
                    },
                },
            };
            return Some(GotoDefinitionResponse::Scalar(location));
        }

        None
    }

    fn find_function_definition(
        &self,
        ast: &Program,
        name: &str,
    ) -> Option<stonescript_parser::ast::Span> {
        for stmt in &ast.statements {
            if let Statement::FunctionDefinition {
                name: func_name,
                span,
                ..
            } = stmt
            {
                if func_name == name {
                    return Some(*span);
                }
            }
        }
        None
    }

    fn find_variable_definition(
        &self,
        ast: &Program,
        name: &str,
    ) -> Option<stonescript_parser::ast::Span> {
        for stmt in &ast.statements {
            match stmt {
                Statement::Assignment { target, span, .. } => {
                    if let Expression::Identifier(var_name, _) = target {
                        if var_name == name {
                            return Some(*span);
                        }
                    }
                }
                Statement::Command {
                    name: cmd_name,
                    args,
                    span,
                    ..
                } => {
                    // Handle var declarations
                    if cmd_name == "var" && !args.is_empty() {
                        if let Expression::Identifier(var_name, _) = &args[0] {
                            if var_name == name {
                                return Some(*span);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// Find variable definition considering scope hierarchy
    /// Searches in the correct scope order: current -> parent -> global
    fn find_variable_definition_scoped(
        &self,
        ast: &Program,
        name: &str,
        pos: AstPosition,
    ) -> Option<stonescript_parser::ast::Span> {
        // Search recursively through statements, respecting scope hierarchy
        self.find_variable_in_statements_at_position(&ast.statements, name, pos)
    }

    /// Find variable definition in statements, considering the position
    fn find_variable_in_statements_at_position(
        &self,
        statements: &[Statement],
        name: &str,
        pos: AstPosition,
    ) -> Option<stonescript_parser::ast::Span> {
        // First pass: find which statement contains our position
        for stmt in statements {
            if let Some(span) = self.get_statement_span(stmt) {
                if span.contains_position(pos) {
                    // This statement contains our position - search within it first
                    match stmt {
                        Statement::FunctionDefinition {
                            body,
                            params,
                            name: func_name,
                            ..
                        } => {
                            // Search within the function body first
                            if let Some(def_span) =
                                self.find_variable_in_statements_at_position(body, name, pos)
                            {
                                return Some(def_span);
                            }

                            // Check function parameters
                            for param in params {
                                if param == name {
                                    // Return the function definition span
                                    return Some(span);
                                }
                            }

                            // Not found in function body or params
                            // Don't fall through to search in other functions
                            // Return None immediately
                        }
                        Statement::Condition {
                            then_block,
                            else_ifs,
                            else_block,
                            ..
                        } => {
                            // Check which block contains the position
                            for s in then_block {
                                if let Some(s_span) = self.get_statement_span(s) {
                                    if s_span.contains_position(pos) || s_span.start.line > pos.line
                                    {
                                        if let Some(def_span) = self
                                            .find_variable_in_statements_at_position(
                                                then_block, name, pos,
                                            )
                                        {
                                            return Some(def_span);
                                        }
                                        break;
                                    }
                                }
                            }

                            for else_if in else_ifs {
                                if else_if.span.contains_position(pos) {
                                    if let Some(def_span) = self
                                        .find_variable_in_statements_at_position(
                                            &else_if.block,
                                            name,
                                            pos,
                                        )
                                    {
                                        return Some(def_span);
                                    }
                                    break;
                                }
                            }

                            if let Some(else_body) = else_block {
                                if let Some(def_span) = self
                                    .find_variable_in_statements_at_position(else_body, name, pos)
                                {
                                    return Some(def_span);
                                }
                            }
                        }
                        Statement::For { body, .. }
                        | Statement::While { body, .. }
                        | Statement::ForIn { body, .. } => {
                            if let Some(def_span) =
                                self.find_variable_in_statements_at_position(body, name, pos)
                            {
                                return Some(def_span);
                            }
                        }
                        _ => {}
                    }

                    // Found the containing statement
                    // For FunctionDefinition, we already returned or fell through
                    // For other statements, search in current scope
                    match stmt {
                        Statement::FunctionDefinition { .. } => {
                            // Already searched inside, not found
                            // Don't search other functions at this level
                            return None;
                        }
                        _ => {
                            // Not a function, can search in current scope
                            break;
                        }
                    }
                }
            }
        }

        // Second pass: search for variable definitions in the current scope
        // that appear before the position
        // This is only reached if we're NOT inside a FunctionDefinition
        self.find_variable_before_position(statements, name, pos)
    }

    /// Find the statement that contains the given position
    fn find_containing_statement<'a>(
        &self,
        statements: &'a [Statement],
        pos: AstPosition,
    ) -> Option<&'a Statement> {
        for stmt in statements {
            if self.statement_contains_position(stmt, pos) {
                return Some(stmt);
            }
        }
        None
    }

    /// Check if a statement contains the position (including nested statements)
    fn statement_contains_position(&self, stmt: &Statement, pos: AstPosition) -> bool {
        match stmt {
            Statement::Assignment { span, .. }
            | Statement::Command { span, .. }
            | Statement::ExpressionStatement { span, .. }
            | Statement::FunctionDefinition { span, .. }
            | Statement::Return { span, .. }
            | Statement::Condition { span, .. }
            | Statement::For { span, .. }
            | Statement::ForIn { span, .. }
            | Statement::While { span, .. }
            | Statement::Output { span, .. }
            | Statement::Import { span, .. }
            | Statement::Comment(_, span) => span.contains_position(pos),
            Statement::Empty => false,
        }
    }

    /// Find variable definition within a specific scope (statement and its nested blocks)
    fn find_variable_in_scope(
        &self,
        stmt: &Statement,
        name: &str,
        pos: AstPosition,
    ) -> Option<stonescript_parser::ast::Span> {
        match stmt {
            Statement::FunctionDefinition { body, params, .. } => {
                // Check if the position is inside this function
                if !self.statement_contains_position(stmt, pos) {
                    return None;
                }

                // Search backwards from the position within the function body
                if let Some(span) = self.find_variable_before_position(body, name, pos) {
                    return Some(span);
                }

                // Check function parameters
                for param in params {
                    if param == name {
                        // Return the function definition span as a fallback
                        // In a better implementation, we'd track parameter spans
                        if let Statement::FunctionDefinition { span, .. } = stmt {
                            return Some(*span);
                        }
                    }
                }

                None
            }
            Statement::Condition {
                then_block,
                else_ifs,
                else_block,
                span,
                ..
            } => {
                if !span.contains_position(pos) {
                    return None;
                }

                // Determine which block contains the position
                for s in then_block {
                    if self.statement_contains_position(s, pos) {
                        // Search within this block first
                        if let Some(def_span) = self.find_variable_in_scope(s, name, pos) {
                            return Some(def_span);
                        }
                        // Then search in the then_block before this statement
                        return self.find_variable_before_position(then_block, name, pos);
                    }
                }

                for else_if in else_ifs {
                    for s in &else_if.block {
                        if self.statement_contains_position(s, pos) {
                            if let Some(def_span) = self.find_variable_in_scope(s, name, pos) {
                                return Some(def_span);
                            }
                            return self.find_variable_before_position(&else_if.block, name, pos);
                        }
                    }
                }

                if let Some(else_body) = else_block {
                    for s in else_body {
                        if self.statement_contains_position(s, pos) {
                            if let Some(def_span) = self.find_variable_in_scope(s, name, pos) {
                                return Some(def_span);
                            }
                            return self.find_variable_before_position(else_body, name, pos);
                        }
                    }
                }

                None
            }
            Statement::For { body, span, .. } | Statement::While { body, span, .. } => {
                if !span.contains_position(pos) {
                    return None;
                }

                for s in body {
                    if self.statement_contains_position(s, pos) {
                        if let Some(def_span) = self.find_variable_in_scope(s, name, pos) {
                            return Some(def_span);
                        }
                        return self.find_variable_before_position(body, name, pos);
                    }
                }

                None
            }
            Statement::ForIn { body, span, .. } => {
                if !span.contains_position(pos) {
                    return None;
                }

                for s in body {
                    if self.statement_contains_position(s, pos) {
                        if let Some(def_span) = self.find_variable_in_scope(s, name, pos) {
                            return Some(def_span);
                        }
                        return self.find_variable_before_position(body, name, pos);
                    }
                }

                None
            }
            _ => None,
        }
    }

    /// Find variable definition before a given position in a list of statements
    /// Only searches at the current level - does NOT recurse into functions
    fn find_variable_before_position(
        &self,
        statements: &[Statement],
        name: &str,
        pos: AstPosition,
    ) -> Option<stonescript_parser::ast::Span> {
        let mut last_match = None;

        for stmt in statements {
            // Stop if we've passed the position
            if let Some(stmt_span) = self.get_statement_span(stmt) {
                if stmt_span.start.line > pos.line
                    || (stmt_span.start.line == pos.line && stmt_span.start.column > pos.column)
                {
                    break;
                }
            }

            // Check if this statement defines the variable
            // IMPORTANT: Do NOT recurse into FunctionDefinition - variables in other functions are not visible
            match stmt {
                Statement::Assignment { target, span, .. } => {
                    if let Expression::Identifier(var_name, _) = target {
                        if var_name == name {
                            last_match = Some(*span);
                        }
                    }
                }
                Statement::Command {
                    name: cmd_name,
                    args,
                    span,
                    ..
                } => {
                    if cmd_name == "var" && !args.is_empty() {
                        if let Expression::Identifier(var_name, _) = &args[0] {
                            if var_name == name {
                                last_match = Some(*span);
                            }
                        }
                    }
                }
                Statement::FunctionDefinition { .. } => {
                    // Skip function definitions - variables inside are in different scope
                    // This prevents finding variables from other functions
                }
                _ => {}
            }
        }

        last_match
    }

    /// Get the span of a statement
    fn get_statement_span(&self, stmt: &Statement) -> Option<stonescript_parser::ast::Span> {
        match stmt {
            Statement::Assignment { span, .. }
            | Statement::Command { span, .. }
            | Statement::ExpressionStatement { span, .. }
            | Statement::FunctionDefinition { span, .. }
            | Statement::Return { span, .. }
            | Statement::Condition { span, .. }
            | Statement::For { span, .. }
            | Statement::ForIn { span, .. }
            | Statement::While { span, .. }
            | Statement::Output { span, .. }
            | Statement::Import { span, .. }
            | Statement::Comment(_, span) => Some(*span),
            Statement::Empty => None,
        }
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
            Statement::FunctionDefinition { body, span, .. } => {
                if span.contains_position(pos) {
                    for s in body {
                        if let Some(expr) = self.find_expression_in_statement(s, pos) {
                            return Some(expr);
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
            Expression::Property { object, .. } => {
                if let Some(inner) = self.find_deepest_expression(object, pos) {
                    Some(inner)
                } else {
                    Some(expr)
                }
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
            _ => Some(expr),
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

impl Default for DefinitionProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use stonescript_parser::parse;
    use tower_lsp::lsp_types::Url;

    #[test]
    fn test_slbutton_scope_issue_from_chisel() {
        // Тест для проблемы: при клике на SLButton на строке 856 в Chisel.txt,
        // выделяется LoadAllLayers на строке 844 вместо SLButton на строке 850

        // Найти путь к файлу Chisel.txt
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.pop(); // выйти из crates/stonescript-lsp
        path.pop(); // выйти из crates
        path.push("test_scripts");
        path.push("Chisel.txt");

        let source =
            fs::read_to_string(&path).expect(&format!("Failed to read Chisel.txt from {:?}", path));

        let ast = parse(&source).expect("Failed to parse Chisel.txt");
        let provider = DefinitionProvider::new();
        let scope = crate::utils::ScopeAnalyzer::new();
        let uri = Url::parse("file:///test/Chisel.txt").unwrap();

        // Клик на SLButton на строке 856 (нумерация с 1, в коде с 0, поэтому 855)
        // Это строка "  SLButton.h = BHeight" (2 пробела отступа)
        let position = Position {
            line: 855,    // строка 856 в редакторе (индексация с 0)
            character: 4, // начало "SLButton" после отступа (2 пробела + 2 отступа от парсера)
        };

        let result = provider.provide_definition(&ast, position, &source, &scope, &uri);

        println!("\n=== Test result ===");
        match &result {
            Some(GotoDefinitionResponse::Scalar(location)) => {
                println!(
                    "Found definition at line {} (0-based), file line {} (1-based)",
                    location.range.start.line,
                    location.range.start.line + 1
                );
                println!(
                    "Expected: line 849 (0-based) = file line 850 (var SLButton = ui.AddButton())"
                );

                if location.range.start.line == 849 {
                    println!("✓ CORRECT: Found SLButton definition");
                } else {
                    println!(
                        "✗ WRONG: Found wrong definition at line {}",
                        location.range.start.line
                    );
                    println!("This is likely LoadAllLayers or another variable");
                }
            }
            None => {
                println!("✗ ERROR: No definition found");
            }
            _ => {
                println!("Unexpected response type");
            }
        }

        // Check if we got the right result
        if let Some(GotoDefinitionResponse::Scalar(ref location)) = result {
            assert_eq!(
                location.range.start.line, 849,
                "Should find SLButton at line 849 (0-based), but found line {}",
                location.range.start.line
            );
        } else {
            panic!("Should find definition for SLButton");
        }
    }

    #[test]
    fn test_variable_scope_isolation() {
        // Тест что переменные в разных функциях изолированы
        // Этот тест работает правильно, так как парсер корректно разделяет эти простые функции
        let source = r#"func FirstFunc()
  var sameName = "first"
  > sameName

func SecondFunc()
  var sameName = "second"
  > sameName
"#;

        let ast = parse(source).expect("Failed to parse");
        let provider = DefinitionProvider::new();
        let scope = crate::utils::ScopeAnalyzer::new();
        let uri = Url::parse("file:///test.txt").unwrap();

        // Клик на sameName во второй функции (строка 6)
        let position = Position {
            line: 6,
            character: 4,
        };

        let result = provider.provide_definition(&ast, position, source, &scope, &uri);

        assert!(
            result.is_some(),
            "Should find definition for sameName in SecondFunc"
        );
        if let Some(GotoDefinitionResponse::Scalar(ref location)) = result {
            // Должно указывать на строку 5 (var sameName = "second")
            // а НЕ на строку 1 (var sameName = "first")
            assert_eq!(
                location.range.start.line, 5,
                "Should find sameName in SecondFunc (line 5), not in FirstFunc (line 1)"
            );
        }
    }

    #[test]
    fn test_variable_in_same_function() {
        // Тест что переменные находятся в той же функции
        let source = r#"func TestFunc()
  var first = 1
  var second = 2
  var third = first + second
  > third
"#;

        let ast = parse(source).expect("Failed to parse");
        let provider = DefinitionProvider::new();
        let scope = crate::utils::ScopeAnalyzer::new();
        let uri = Url::parse("file:///test.txt").unwrap();

        // Клик на third в последней строке
        let position = Position {
            line: 4,
            character: 4,
        };

        let result = provider.provide_definition(&ast, position, source, &scope, &uri);

        assert!(result.is_some(), "Should find definition for third");
        if let Some(GotoDefinitionResponse::Scalar(ref location)) = result {
            assert_eq!(
                location.range.start.line, 3,
                "Should find var third at line 3"
            );
        }
    }
}
