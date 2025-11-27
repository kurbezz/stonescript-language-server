//! Signature help provider with comprehensive function signature information

use crate::data::*;
use crate::utils::ScopeAnalyzer;
use stonescript_parser::ast::{Expression, Position as AstPosition, Program, Statement};
use tower_lsp::lsp_types::*;

pub struct SignatureHelpProvider;

impl SignatureHelpProvider {
    pub fn new() -> Self {
        Self
    }

    pub fn provide_signature_help(
        &self,
        ast: &Program,
        position: Position,
        source: &str,
        scope: &ScopeAnalyzer,
    ) -> Option<SignatureHelp> {
        // Try AST-based approach first
        let ast_pos = AstPosition::new(position.line as usize, position.character as usize);
        if let Some(help) = self.find_signature_from_ast(ast, ast_pos, source, scope) {
            return Some(help);
        }

        // Fallback to text-based approach
        self.find_signature_from_text(position, source, scope)
    }

    fn find_signature_from_ast(
        &self,
        ast: &Program,
        pos: AstPosition,
        source: &str,
        scope: &ScopeAnalyzer,
    ) -> Option<SignatureHelp> {
        // Find function call at position
        for stmt in &ast.statements {
            if let Some(help) = self.find_signature_in_statement(stmt, pos, source, scope) {
                return Some(help);
            }
        }
        None
    }

    fn find_signature_in_statement(
        &self,
        stmt: &Statement,
        pos: AstPosition,
        source: &str,
        scope: &ScopeAnalyzer,
    ) -> Option<SignatureHelp> {
        match stmt {
            Statement::ExpressionStatement { expression, .. } => {
                self.find_signature_in_expression(expression, pos, source, scope)
            }
            Statement::Assignment { value, .. } => {
                self.find_signature_in_expression(value, pos, source, scope)
            }
            Statement::Condition {
                condition,
                then_block,
                else_ifs,
                else_block,
                ..
            } => {
                if let Some(help) = self.find_signature_in_expression(condition, pos, source, scope)
                {
                    return Some(help);
                }
                for s in then_block {
                    if let Some(help) = self.find_signature_in_statement(s, pos, source, scope) {
                        return Some(help);
                    }
                }
                for elif in else_ifs {
                    if let Some(help) =
                        self.find_signature_in_expression(&elif.condition, pos, source, scope)
                    {
                        return Some(help);
                    }
                    for s in &elif.block {
                        if let Some(help) = self.find_signature_in_statement(s, pos, source, scope)
                        {
                            return Some(help);
                        }
                    }
                }
                if let Some(else_stmts) = else_block {
                    for s in else_stmts {
                        if let Some(help) = self.find_signature_in_statement(s, pos, source, scope)
                        {
                            return Some(help);
                        }
                    }
                }
                None
            }
            Statement::Return { value, .. } => {
                if let Some(expr) = value {
                    self.find_signature_in_expression(expr, pos, source, scope)
                } else {
                    None
                }
            }
            Statement::For { range, body, .. } => {
                if let Some(help) = self.find_signature_in_expression(&range.0, pos, source, scope)
                {
                    return Some(help);
                }
                if let Some(help) = self.find_signature_in_expression(&range.1, pos, source, scope)
                {
                    return Some(help);
                }
                for s in body {
                    if let Some(help) = self.find_signature_in_statement(s, pos, source, scope) {
                        return Some(help);
                    }
                }
                None
            }
            Statement::Command { args, .. } => {
                for arg in args {
                    if let Some(help) = self.find_signature_in_expression(arg, pos, source, scope) {
                        return Some(help);
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn find_signature_in_expression(
        &self,
        expr: &Expression,
        pos: AstPosition,
        source: &str,
        scope: &ScopeAnalyzer,
    ) -> Option<SignatureHelp> {
        match expr {
            Expression::FunctionCall {
                function,
                args,
                span,
            } => {
                // Check if cursor is inside this function call
                if span.contains_position(pos) {
                    // Determine active parameter
                    let active_param = self.calculate_active_parameter(args, pos, source);

                    // Get function signature
                    return self.get_signature_for_function(function, scope, active_param);
                }

                // Check nested expressions
                if let Some(help) = self.find_signature_in_expression(function, pos, source, scope)
                {
                    return Some(help);
                }
                for arg in args {
                    if let Some(help) = self.find_signature_in_expression(arg, pos, source, scope) {
                        return Some(help);
                    }
                }
                None
            }
            Expression::BinaryOp { left, right, .. } => {
                if let Some(help) = self.find_signature_in_expression(left, pos, source, scope) {
                    Some(help)
                } else {
                    self.find_signature_in_expression(right, pos, source, scope)
                }
            }
            Expression::UnaryOp { operand, .. } => {
                self.find_signature_in_expression(operand, pos, source, scope)
            }
            Expression::Property { object, .. } => {
                self.find_signature_in_expression(object, pos, source, scope)
            }
            Expression::IndexAccess { object, index, .. } => {
                if let Some(help) = self.find_signature_in_expression(object, pos, source, scope) {
                    Some(help)
                } else {
                    self.find_signature_in_expression(index, pos, source, scope)
                }
            }
            Expression::Array { elements, .. } => {
                for elem in elements {
                    if let Some(help) = self.find_signature_in_expression(elem, pos, source, scope)
                    {
                        return Some(help);
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn calculate_active_parameter(
        &self,
        args: &[Expression],
        pos: AstPosition,
        _source: &str,
    ) -> Option<u32> {
        // Find which argument the cursor is in
        for (idx, arg) in args.iter().enumerate() {
            if arg.span().contains_position(pos) {
                return Some(idx as u32);
            }
        }

        // If cursor is after all arguments, it's on the next parameter
        if !args.is_empty() {
            let last_arg = &args[args.len() - 1];
            if pos.line > last_arg.span().end.line
                || (pos.line == last_arg.span().end.line
                    && pos.column >= last_arg.span().end.column)
            {
                return Some(args.len() as u32);
            }
        }

        Some(0)
    }

    fn get_signature_for_function(
        &self,
        function: &Expression,
        _scope: &ScopeAnalyzer,
        active_param: Option<u32>,
    ) -> Option<SignatureHelp> {
        match function {
            Expression::Property {
                object, property, ..
            } => {
                // Try to identify namespace.function pattern
                if let Expression::Identifier(namespace, _) = object.as_ref() {
                    // Check built-in functions
                    if let Some(func) = get_function(property) {
                        if func.namespace == namespace {
                            return Some(self.create_signature_help(func, active_param));
                        }
                    }

                    // Check all namespaces
                    let all_funcs = [
                        MATH_FUNCTIONS,
                        STRING_FUNCTIONS,
                        STORAGE_FUNCTIONS,
                        MUSIC_FUNCTIONS,
                        UI_FUNCTIONS,
                    ]
                    .concat();

                    for func in &all_funcs {
                        if func.namespace == namespace && func.name == property {
                            return Some(self.create_signature_help(func, active_param));
                        }
                    }

                    // Check UI methods
                    // Note: UI_METHODS is just a list of strings, not full signatures
                    // Full UI method signatures would need to be defined separately
                }
                None
            }
            Expression::Identifier(name, _) => {
                // Check built-in functions without namespace
                if let Some(func) = get_function(name) {
                    return Some(self.create_signature_help(func, active_param));
                }

                None
            }
            _ => None,
        }
    }

    fn create_signature_help(
        &self,
        func: &FunctionSignature,
        active_param: Option<u32>,
    ) -> SignatureHelp {
        let params: Vec<ParameterInformation> = func
            .parameters
            .iter()
            .map(|p| {
                let optional_marker = if p.optional { "?" } else { "" };
                ParameterInformation {
                    label: ParameterLabel::Simple(format!(
                        "{}{}: {}",
                        p.name, optional_marker, p.typ
                    )),
                    documentation: None,
                }
            })
            .collect();

        let label = if func.namespace.is_empty() {
            format!(
                "func {}({}) -> {}",
                func.name,
                func.parameters
                    .iter()
                    .map(|p| {
                        let optional_marker = if p.optional { "?" } else { "" };
                        format!("{}{}: {}", p.name, optional_marker, p.typ)
                    })
                    .collect::<Vec<_>>()
                    .join(", "),
                func.return_type
            )
        } else {
            format!(
                "func {}.{}({}) -> {}",
                func.namespace,
                func.name,
                func.parameters
                    .iter()
                    .map(|p| {
                        let optional_marker = if p.optional { "?" } else { "" };
                        format!("{}{}: {}", p.name, optional_marker, p.typ)
                    })
                    .collect::<Vec<_>>()
                    .join(", "),
                func.return_type
            )
        };

        SignatureHelp {
            signatures: vec![SignatureInformation {
                label,
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: func.description.to_string(),
                })),
                parameters: Some(params),
                active_parameter: None,
            }],
            active_signature: Some(0),
            active_parameter: active_param,
        }
    }

    fn find_signature_from_text(
        &self,
        position: Position,
        source: &str,
        _scope: &ScopeAnalyzer,
    ) -> Option<SignatureHelp> {
        let line = source.lines().nth(position.line as usize)?;
        let text_before = &line[..position.character.min(line.len() as u32) as usize];

        // Count commas to determine active parameter
        let active_param = text_before
            .chars()
            .rev()
            .take_while(|&c| c != '(')
            .filter(|&c| c == ',')
            .count() as u32;

        // Look for pattern: namespace.function( or function(
        let namespace_pattern = regex::Regex::new(r"(\w+)\.(\w+)\s*\([^)]*$").ok()?;
        if let Some(caps) = namespace_pattern.captures(text_before) {
            let namespace = caps.get(1)?.as_str();
            let func_name = caps.get(2)?.as_str();

            // Try to find the function
            let all_funcs = [
                MATH_FUNCTIONS,
                STRING_FUNCTIONS,
                STORAGE_FUNCTIONS,
                MUSIC_FUNCTIONS,
                UI_FUNCTIONS,
            ]
            .concat();

            for func in &all_funcs {
                if func.namespace == namespace && func.name == func_name {
                    return Some(self.create_signature_help(func, Some(active_param)));
                }
            }

            // Note: UI_METHODS is just a list of strings, not full signatures
            // Full UI method signatures would need to be defined separately
        }

        // Look for simple function call
        let simple_pattern = regex::Regex::new(r"(\w+)\s*\([^)]*$").ok()?;
        if let Some(caps) = simple_pattern.captures(text_before) {
            let func_name = caps.get(1)?.as_str();

            // Check built-in functions
            if let Some(func) = get_function(func_name) {
                return Some(self.create_signature_help(func, Some(active_param)));
            }
        }

        None
    }
}

impl Default for SignatureHelpProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_help_provider() {
        let provider = SignatureHelpProvider::new();
        assert!(provider
            .find_signature_from_text(Position::new(0, 10), "math.Abs(", &ScopeAnalyzer::new())
            .is_some());
    }

    #[test]
    fn test_active_parameter_counting() {
        let provider = SignatureHelpProvider::new();
        let scope = ScopeAnalyzer::new();

        // Test with one comma (second parameter)
        let result =
            provider.find_signature_from_text(Position::new(0, 15), "math.Pow(2, ", &scope);
        assert!(result.is_some());
        if let Some(help) = result {
            assert_eq!(help.active_parameter, Some(1));
        }
    }
}
