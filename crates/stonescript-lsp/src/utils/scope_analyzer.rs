//! Scope analysis using nom-based AST

use crate::data::Type;
use std::collections::HashMap;
use stonescript_parser::ast::{Expression, Position, Program, Span, Statement};

/// A variable in scope
#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub scope_id: usize,
    pub inferred_type: Type,
    pub definition_span: Option<Span>,
}

/// Scope information
#[derive(Debug)]
pub struct Scope {
    pub id: usize,
    pub parent: Option<usize>,
    pub variables: HashMap<String, Variable>,
    pub span: Option<Span>,
}

/// Analyzer for variable scopes
pub struct ScopeAnalyzer {
    scopes: Vec<Scope>,
    next_scope_id: usize,
    current_scope: usize,
    functions: HashMap<String, FunctionStub>,
}

impl ScopeAnalyzer {
    pub fn new() -> Self {
        // Global scope
        let global_scope = Scope {
            id: 0,
            parent: None,
            variables: HashMap::new(),
            span: None,
        };

        Self {
            scopes: vec![global_scope],
            next_scope_id: 1,
            current_scope: 0,
            functions: HashMap::new(),
        }
    }

    /// Analyze a nom-based AST
    pub fn analyze_ast(&mut self, program: &Program) {
        for statement in &program.statements {
            self.analyze_statement(statement);
        }
    }

    fn analyze_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::ForIn {
                variable,
                collection,
                body,
                ..
            } => {
                self.analyze_expression(collection);
                let _scope = self.enter_scope();
                self.add_variable(variable.clone());
                for stmt in body {
                    self.analyze_statement(stmt);
                }
                self.exit_scope();
            }
            Statement::Assignment { target, value, .. } => {
                // Extract variable name from target and add to current scope
                if let Expression::Identifier(name, _) = target {
                    // Try to infer type from value
                    let inferred_type = crate::utils::type_inference::infer_type(value);
                    self.add_variable_with_type(name.clone(), inferred_type);
                }
                // Analyze both target and value expressions
                self.analyze_expression(target);
                self.analyze_expression(value);
            }
            Statement::Command { name, args, .. } => {
                // Handle var declarations (var varname)
                if name == "var" && !args.is_empty() {
                    if let Expression::Identifier(var_name, _) = &args[0] {
                        self.add_variable(var_name.clone());
                    }
                }
                // Analyze arguments
                for arg in args {
                    self.analyze_expression(arg);
                }
            }
            Statement::Condition {
                condition,
                then_block,
                else_ifs,
                else_block,
                ..
            } => {
                self.analyze_expression(condition);

                // Analyze then block in new scope
                let _then_scope = self.enter_scope();
                for stmt in then_block {
                    self.analyze_statement(stmt);
                }
                self.exit_scope();

                // Analyze else-if blocks
                for else_if in else_ifs {
                    self.analyze_expression(&else_if.condition);
                    let _else_if_scope = self.enter_scope();
                    for stmt in &else_if.block {
                        self.analyze_statement(stmt);
                    }
                    self.exit_scope();
                }

                // Analyze else block
                if let Some(stmts) = else_block {
                    let _else_scope = self.enter_scope();
                    for stmt in stmts {
                        self.analyze_statement(stmt);
                    }
                    self.exit_scope();
                }
            }
            Statement::Output { position, text, .. } => {
                if let Some((x, y)) = position {
                    self.analyze_expression(x);
                    self.analyze_expression(y);
                }
                self.analyze_expression(text);
            }
            Statement::ExpressionStatement { expression, .. } => {
                self.analyze_expression(expression);
            }
            Statement::FunctionDefinition {
                name, params, body, ..
            } => {
                self.functions.insert(
                    name.clone(),
                    FunctionStub {
                        name: name.clone(),
                        parameters: params.clone(),
                    },
                );
                self.add_variable(name.clone());
                let _scope = self.enter_scope();
                for param in params {
                    self.add_variable(param.clone());
                }
                for stmt in body {
                    self.analyze_statement(stmt);
                }
                self.exit_scope();
            }
            Statement::Return { value, .. } => {
                if let Some(expr) = value {
                    self.analyze_expression(expr);
                }
            }
            Statement::For {
                variable,
                range,
                body,
                ..
            } => {
                self.analyze_expression(&range.0);
                self.analyze_expression(&range.1);
                let _scope = self.enter_scope();
                self.add_variable(variable.clone());
                for stmt in body {
                    self.analyze_statement(stmt);
                }
                self.exit_scope();
            }
            Statement::While {
                condition, body, ..
            } => {
                self.analyze_expression(condition);
                let _scope = self.enter_scope();
                for stmt in body {
                    self.analyze_statement(stmt);
                }
                self.exit_scope();
            }
            Statement::Import { .. } => {
                // Import statements don't affect scope
            }
            Statement::Comment(_, _) | Statement::Empty => {
                // Nothing to analyze
            }
        }
    }

    fn analyze_expression(&mut self, expression: &Expression) {
        match expression {
            Expression::Boolean(_, _) => {
                // Boolean literals don't need scope analysis
            }
            Expression::Identifier(name, _) => {
                // Reference to a variable - could track usage here
                let _ = self.find_variable(name);
            }
            Expression::Property { object, .. } => {
                self.analyze_expression(object);
            }
            Expression::FunctionCall { function, args, .. } => {
                self.analyze_expression(function);
                for arg in args {
                    self.analyze_expression(arg);
                }
            }
            Expression::BinaryOp { left, right, .. } => {
                self.analyze_expression(left);
                self.analyze_expression(right);
            }
            Expression::UnaryOp { operand, .. } => {
                self.analyze_expression(operand);
            }
            Expression::Interpolation(parts, _) => {
                for part in parts {
                    if let stonescript_parser::ast::InterpolationPart::Expression(expr) = part {
                        self.analyze_expression(expr);
                    }
                }
            }
            Expression::New { .. } => {
                // Object instantiation - nothing to analyze
            }
            Expression::Array { elements, .. } => {
                // Analyze array elements
                for element in elements {
                    self.analyze_expression(element);
                }
            }
            Expression::IndexAccess { object, index, .. } => {
                // Analyze both object and index expressions
                self.analyze_expression(object);
                self.analyze_expression(index);
            }
            Expression::Integer(_, _) | Expression::Float(_, _) | Expression::String(_, _) => {
                // Literals don't need scope analysis
            }
        }
    }

    fn add_variable(&mut self, name: String) {
        self.add_variable_with_type(name, Type::Unknown);
    }

    fn add_variable_with_type(&mut self, name: String, inferred_type: Type) {
        let variable = Variable {
            name: name.clone(),
            scope_id: self.current_scope,
            inferred_type,
            definition_span: None,
        };

        self.scopes[self.current_scope]
            .variables
            .insert(name, variable);
    }

    fn enter_scope(&mut self) -> usize {
        let scope_id = self.next_scope_id;
        self.next_scope_id += 1;

        let scope = Scope {
            id: scope_id,
            parent: Some(self.current_scope),
            variables: HashMap::new(),
            span: None,
        };

        self.scopes.push(scope);
        self.current_scope = scope_id;
        scope_id
    }

    fn exit_scope(&mut self) {
        if let Some(parent) = self.scopes[self.current_scope].parent {
            self.current_scope = parent;
        }
    }

    /// Find a variable in current or parent scopes
    pub fn find_variable(&self, name: &str) -> Option<&Variable> {
        let mut current = self.current_scope;
        loop {
            if let Some(var) = self.scopes[current].variables.get(name) {
                return Some(var);
            }

            match self.scopes[current].parent {
                Some(parent) => current = parent,
                None => return None,
            }
        }
    }

    /// Get all variables in current scope
    pub fn get_variables_in_scope(&self, scope_id: usize) -> Vec<&Variable> {
        let mut variables = Vec::new();
        let mut current = scope_id;

        loop {
            for var in self.scopes[current].variables.values() {
                variables.push(var);
            }

            match self.scopes[current].parent {
                Some(parent) => current = parent,
                None => break,
            }
        }

        variables
    }

    /// Get all variables in the global scope
    pub fn get_all_variables(&self) -> Vec<&Variable> {
        self.scopes
            .iter()
            .flat_map(|scope| scope.variables.values())
            .collect()
    }

    /// Check if a variable exists in any scope
    pub fn has_variable(&self, name: &str) -> bool {
        self.scopes
            .iter()
            .any(|scope| scope.variables.contains_key(name))
    }

    /// Find variables at a specific position (compatibility method)
    /// Note: byte_offset is ignored in AST-based analysis - returns all variables
    /// This is a compatibility shim for code migrating from tree-sitter
    pub fn find_variables_at(&self, _byte_offset: usize) -> Vec<&Variable> {
        self.get_all_variables()
    }

    /// Get user-defined functions (compatibility method)
    /// Note: StoneScript doesn't support user-defined functions, returns empty vec
    /// This is a compatibility shim for code migrating from tree-sitter
    pub fn get_functions(&self) -> Vec<FunctionStub> {
        self.functions.values().cloned().collect()
    }

    pub fn find_function(&self, name: &str) -> Option<&FunctionStub> {
        self.functions.get(name)
    }
}

/// Stub type for function compatibility
/// StoneScript doesn't have user-defined functions, but some code still references them
#[derive(Debug, Clone)]
pub struct FunctionStub {
    pub name: String,
    pub parameters: Vec<String>,
}

impl Default for ScopeAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stonescript_parser::parse_source;

    #[test]
    fn test_basic_variable() {
        let source = "var x\nx = 10";
        let program = parse_source(source).unwrap();

        let mut analyzer = ScopeAnalyzer::new();
        analyzer.analyze_ast(&program);

        assert!(analyzer.has_variable("x"));
    }

    #[test]
    fn test_scoped_variables() {
        let source = r#"
var x
?hp < 10
  var y
  y = 5
"#;
        let program = parse_source(source).unwrap();

        let mut analyzer = ScopeAnalyzer::new();
        analyzer.analyze_ast(&program);

        assert!(analyzer.has_variable("x"));
        assert!(analyzer.has_variable("y"));
    }

    #[test]
    fn test_multiple_scopes() {
        let source = r#"
var a
?loc = caves
  var b
:
  var c
"#;
        let program = parse_source(source).unwrap();

        let mut analyzer = ScopeAnalyzer::new();
        analyzer.analyze_ast(&program);

        assert!(analyzer.has_variable("a"));
        assert!(analyzer.has_variable("b"));
        assert!(analyzer.has_variable("c"));
    }
}
