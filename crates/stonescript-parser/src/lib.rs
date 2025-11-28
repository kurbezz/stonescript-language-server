//! StoneScript Parser
//!
//! Parser for the StoneScript language using nom parser combinators

pub mod ast;
pub mod parser;

// Re-export main types and functions
pub use ast::{
    AssignmentOperator, BinaryOperator, ElseIf, Expression, InterpolationPart, Position, Program,
    Span, Statement, UnaryOperator,
};
pub use parser::parse;

/// Parse StoneScript source code and return a Program AST
pub fn parse_source(source: &str) -> Result<Program, String> {
    // Strip UTF-8 BOM if present
    let source = source.strip_prefix('\u{FEFF}').unwrap_or(source);
    parse(source)
}

/// Visitor trait for traversing the AST
pub trait Visitor {
    fn visit_program(&mut self, program: &Program) {
        for statement in &program.statements {
            self.visit_statement(statement);
        }
    }

    fn visit_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Condition {
                condition,
                then_block,
                else_ifs,
                else_block,
                ..
            } => {
                self.visit_condition(condition, then_block, else_ifs, else_block);
            }
            Statement::Command { name, args, .. } => {
                self.visit_command(name, args);
            }
            Statement::Assignment { target, value, .. } => {
                self.visit_assignment(target, value);
            }
            Statement::Output { position, text, .. } => {
                self.visit_output(position, text);
            }

            Statement::ExpressionStatement { expression, .. } => {
                self.visit_expression(expression);
            }
            Statement::FunctionDefinition {
                name, params, body, ..
            } => {
                self.visit_function_definition(name, params, body);
            }
            Statement::Return { value, .. } => {
                self.visit_return(value);
            }
            Statement::For {
                variable,
                range,
                body,
                ..
            } => {
                self.visit_for(variable, range, body);
            }
            Statement::ForIn {
                variable,
                collection,
                body,
                ..
            } => {
                self.visit_for_in(variable, collection, body);
            }
            Statement::While {
                condition, body, ..
            } => {
                self.visit_while(condition, body);
            }
            Statement::Import { path, .. } => {
                self.visit_import(path);
            }
            Statement::Comment(text, _) => {
                self.visit_comment(text);
            }
            Statement::Empty => {
                self.visit_empty();
            }
        }
    }

    fn visit_condition(
        &mut self,
        condition: &Expression,
        then_block: &[Statement],
        else_ifs: &[ElseIf],
        else_block: &Option<Vec<Statement>>,
    ) {
        self.visit_expression(condition);
        for stmt in then_block {
            self.visit_statement(stmt);
        }
        for else_if in else_ifs {
            self.visit_expression(&else_if.condition);
            for stmt in &else_if.block {
                self.visit_statement(stmt);
            }
        }
        if let Some(stmts) = else_block {
            for stmt in stmts {
                self.visit_statement(stmt);
            }
        }
    }

    fn visit_command(&mut self, _name: &str, args: &[Expression]) {
        for arg in args {
            self.visit_expression(arg);
        }
    }

    fn visit_assignment(&mut self, target: &Expression, value: &Expression) {
        self.visit_expression(target);
        self.visit_expression(value);
    }

    fn visit_output(&mut self, position: &Option<(Expression, Expression)>, text: &Expression) {
        if let Some((x, y)) = position {
            self.visit_expression(x);
            self.visit_expression(y);
        }
        self.visit_expression(text);
    }

    fn visit_comment(&mut self, _text: &str) {}

    fn visit_import(&mut self, _path: &str) {}

    fn visit_function_definition(&mut self, _name: &str, _params: &[String], body: &[Statement]) {
        for statement in body {
            self.visit_statement(statement);
        }
    }

    fn visit_return(&mut self, value: &Option<Expression>) {
        if let Some(expr) = value {
            self.visit_expression(expr);
        }
    }

    fn visit_for(&mut self, _variable: &str, range: &(Expression, Expression), body: &[Statement]) {
        self.visit_expression(&range.0);
        self.visit_expression(&range.1);
        for statement in body {
            self.visit_statement(statement);
        }
    }

    fn visit_for_in(&mut self, _variable: &str, collection: &Expression, body: &[Statement]) {
        self.visit_expression(collection);
        for statement in body {
            self.visit_statement(statement);
        }
    }

    fn visit_while(&mut self, condition: &Expression, body: &[Statement]) {
        self.visit_expression(condition);
        for statement in body {
            self.visit_statement(statement);
        }
    }

    fn visit_empty(&mut self) {}

    fn visit_expression(&mut self, expression: &Expression) {
        match expression {
            Expression::Integer(..) => {}
            Expression::Float(..) => {}
            Expression::Boolean(..) => {}
            Expression::String(..) => {}
            Expression::Identifier(..) => {}
            Expression::New { .. } => {}
            Expression::Property { object, .. } => {
                self.visit_expression(object);
            }
            Expression::FunctionCall { function, args, .. } => {
                self.visit_expression(function);
                for arg in args {
                    self.visit_expression(arg);
                }
            }
            Expression::BinaryOp { left, right, .. } => {
                self.visit_expression(left);
                self.visit_expression(right);
            }
            Expression::UnaryOp { operand, .. } => {
                self.visit_expression(operand);
            }
            Expression::Interpolation(parts, _) => {
                for part in parts {
                    if let InterpolationPart::Expression(expr) = part {
                        self.visit_expression(expr);
                    }
                }
            }
            Expression::Array { elements, .. } => {
                for elem in elements {
                    self.visit_expression(elem);
                }
            }
            Expression::IndexAccess { object, index, .. } => {
                self.visit_expression(object);
                self.visit_expression(index);
            }
            _ => {}
        }
    }
}

/// Collect all identifiers (variables) used in the program
pub struct IdentifierCollector {
    pub identifiers: Vec<String>,
}

impl IdentifierCollector {
    pub fn new() -> Self {
        Self {
            identifiers: Vec::new(),
        }
    }

    pub fn collect(program: &Program) -> Vec<String> {
        let mut collector = Self::new();
        collector.visit_program(program);
        collector.identifiers
    }
}

impl Default for IdentifierCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl Visitor for IdentifierCollector {
    fn visit_assignment(&mut self, target: &Expression, value: &Expression) {
        // Extract identifier from target if it's a simple identifier
        if let Expression::Identifier(name, _) = target {
            if !self.identifiers.contains(name) {
                self.identifiers.push(name.clone());
            }
        }
        // Visit both target and value expressions
        self.visit_expression(target);
        self.visit_expression(value);
    }

    fn visit_function_definition(&mut self, name: &str, params: &[String], body: &[Statement]) {
        // Collect function name
        if !self.identifiers.contains(&name.to_string()) {
            self.identifiers.push(name.to_string());
        }
        // Collect parameter names
        for param in params {
            if !self.identifiers.contains(param) {
                self.identifiers.push(param.clone());
            }
        }
        // Visit function body
        for statement in body {
            self.visit_statement(statement);
        }
    }

    fn visit_for(&mut self, variable: &str, range: &(Expression, Expression), body: &[Statement]) {
        // Collect loop variable
        if !self.identifiers.contains(&variable.to_string()) {
            self.identifiers.push(variable.to_string());
        }
        // Visit range expressions
        self.visit_expression(&range.0);
        self.visit_expression(&range.1);
        // Visit loop body
        for statement in body {
            self.visit_statement(statement);
        }
    }

    fn visit_expression(&mut self, expression: &Expression) {
        match expression {
            Expression::Identifier(name, _) => {
                if !self.identifiers.contains(name) {
                    self.identifiers.push(name.clone());
                }
            }
            Expression::Property {
                object, property, ..
            } => {
                self.visit_expression(object);
                if !self.identifiers.contains(property) {
                    self.identifiers.push(property.clone());
                }
            }
            Expression::New { .. } => {}
            Expression::Array { elements, .. } => {
                for element in elements {
                    self.visit_expression(element);
                }
            }
            Expression::IndexAccess { object, index, .. } => {
                self.visit_expression(object);
                self.visit_expression(index);
            }
            _ => {
                // Continue visiting nested expressions
                match expression {
                    Expression::FunctionCall { function, args, .. } => {
                        self.visit_expression(function);
                        for arg in args {
                            self.visit_expression(arg);
                        }
                    }
                    Expression::BinaryOp { left, right, .. } => {
                        self.visit_expression(left);
                        self.visit_expression(right);
                    }
                    Expression::UnaryOp { operand, .. } => {
                        self.visit_expression(operand);
                    }
                    Expression::Interpolation(parts, _) => {
                        for part in parts {
                            if let InterpolationPart::Expression(expr) = part {
                                self.visit_expression(&expr);
                            }
                        }
                    }
                    Expression::New { .. } => {}
                    Expression::Array { .. } => {}
                    _ => {}
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_variable() {
        let result = parse_source("var x");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_very_simple_function_call() {
        let result = parse_source("clamp(1,2,3)");
        assert!(
            result.is_ok(),
            "Failed to parse very simple function call: {:?}",
            result
        );
        let program = result.unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_parse_function_call_with_property() {
        let result = parse_source("clamp(input.x,0,1)");
        assert!(
            result.is_ok(),
            "Failed to parse function call with property: {:?}",
            result
        );
        let program = result.unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_parse_subtraction() {
        // Test basic subtraction expression
        let result = parse_source("screenw-1");
        assert!(
            result.is_ok(),
            "Failed to parse subtraction expression: {:?}",
            result
        );
    }

    #[test]
    fn test_parse_subtraction_in_parens() {
        // Test subtraction in parentheses
        let result = parse_source("foo(screenw-1)");
        assert!(
            result.is_ok(),
            "Failed to parse function call with subtraction in parens: {:?}",
            result
        );
    }

    #[test]
    fn test_parse_function_call_with_two_args() {
        // Test with two simple args first
        let result = parse_source("clamp(0,screenw-1)");
        assert!(
            result.is_ok(),
            "Failed to parse function call with two args: {:?}",
            result
        );
    }

    #[test]
    fn test_parse_compound_assignment() {
        let result = parse_source("x += 5");
        assert!(
            result.is_ok(),
            "Failed to parse compound assignment: {:?}",
            result
        );
        let program = result.unwrap();
        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Statement::Assignment {
                target, op, value, ..
            } => {
                match target {
                    Expression::Identifier(name, _) => assert_eq!(name, "x"),
                    _ => panic!("Expected Identifier target"),
                }
                assert_eq!(*op, AssignmentOperator::AddAssign);
                match value {
                    Expression::Integer(5, _) => {}
                    _ => panic!("Expected Integer value"),
                }
            }
            _ => panic!("Expected Assignment statement"),
        }
    }

    #[test]
    fn test_parse_compound_assignment_with_index() {
        let result = parse_source("arr[i] += 1");
        assert!(
            result.is_ok(),
            "Failed to parse compound assignment with index: {:?}",
            result
        );
        let program = result.unwrap();
        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Statement::Assignment { target, op, .. } => {
                match target {
                    Expression::IndexAccess { .. } => {}
                    _ => panic!("Expected IndexAccess target, got {:?}", target),
                }
                assert_eq!(*op, AssignmentOperator::AddAssign);
            }
            _ => panic!("Expected Assignment statement"),
        }
    }

    #[test]
    fn test_parse_simple_condition_with_else() {
        // Test simple condition with else - minimal case
        // Based on actual DragController.txt structure with CRLF
        let source = "func test()\r\n  ?x > 5\r\n    return true\r\n  :\r\n    return false";
        let result = parse_source(source);
        assert!(
            result.is_ok(),
            "Failed to parse simple condition with else: {:?}",
            result
        );
        let program = result.unwrap();
        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Statement::FunctionDefinition { body, .. } => {
                assert_eq!(body.len(), 1);
                match &body[0] {
                    Statement::Condition {
                        then_block,
                        else_block,
                        ..
                    } => {
                        assert_eq!(then_block.len(), 1);
                        assert!(
                            else_block.is_some(),
                            "else_block should be Some, but got None"
                        );
                        if let Some(else_stmts) = else_block {
                            assert_eq!(else_stmts.len(), 1);
                        }
                    }
                    _ => panic!("Expected Condition statement in function body"),
                }
            }
            _ => panic!("Expected FunctionDefinition statement"),
        }
    }

    #[test]
    fn test_parse_multiline_condition_with_continuation() {
        // Test multi-line condition with line continuation (^) and else
        // Note: else (:) should be at same indent as condition (?)
        let source = "?x >= compx &\n^x < (compx + w)\n  return true\n:\n  return false";
        let result = parse_source(source);
        assert!(
            result.is_ok(),
            "Failed to parse multi-line condition with continuation: {:?}",
            result
        );
        let program = result.unwrap();
        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Statement::Condition {
                then_block,
                else_block,
                ..
            } => {
                assert_eq!(then_block.len(), 1);
                assert!(else_block.is_some());
                if let Some(else_stmts) = else_block {
                    assert_eq!(else_stmts.len(), 1);
                }
            }
            _ => panic!("Expected Condition statement"),
        }
    }

    #[test]
    fn test_parse_function_call_with_minus() {
        // Test with simple subtraction first
        let result = parse_source("clamp(0,0,screenw-1)");
        assert!(
            result.is_ok(),
            "Failed to parse function call with subtraction: {:?}",
            result
        );
    }

    #[test]
    fn test_parse_simple_function_call() {
        let result = parse_source("clamp(input.x,0,screenw-1)");
        assert!(
            result.is_ok(),
            "Failed to parse simple function call: {:?}",
            result
        );
        let program = result.unwrap();
        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Statement::ExpressionStatement { expression, .. } => match expression {
                Expression::FunctionCall { .. } => {}
                _ => panic!("Expected FunctionCall, got {:?}", expression),
            },
            _ => panic!(
                "Expected ExpressionStatement, got {:?}",
                &program.statements[0]
            ),
        }
    }

    #[test]
    fn test_parse_assignment_with_function_call() {
        let result = parse_source("ix = clamp(input.x,0,screenw-1)");
        assert!(
            result.is_ok(),
            "Failed to parse assignment with function call: {:?}",
            result
        );
        let program = result.unwrap();
        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Statement::Assignment { target, value, .. } => {
                match target {
                    Expression::Identifier(name, _) => assert_eq!(name, "ix"),
                    _ => panic!("Expected Identifier target"),
                }
                match value {
                    Expression::FunctionCall { .. } => {}
                    _ => panic!("Expected FunctionCall value, got {:?}", value),
                }
            }
            _ => panic!("Expected Assignment statement"),
        }
    }

    #[test]
    fn test_parse_conditional() {
        let source = "?hp < 10\n  activate potion";
        let result = parse_source(source);
        assert!(result.is_ok());

        if let Ok(program) = result {
            assert!(!program.statements.is_empty());
            match &program.statements[0] {
                Statement::Condition { .. } => {
                    // Success
                }
                _ => panic!("Expected conditional statement"),
            }
        }
    }

    #[test]
    fn test_parse_command() {
        let source = "equip sword";
        let result = parse_source(source);
        assert!(result.is_ok());

        if let Ok(program) = result {
            assert_eq!(program.statements.len(), 1);
            match &program.statements[0] {
                Statement::Command { name, args, .. } => {
                    assert_eq!(name, "equip");
                    assert_eq!(args.len(), 1);
                }
                _ => panic!("Expected command statement"),
            }
        }
    }

    #[test]
    fn test_identifier_collector() {
        let source = "var x\nx = loc.stars";
        let program = parse_source(source).unwrap();
        let identifiers = IdentifierCollector::collect(&program);

        assert!(identifiers.contains(&"x".to_string()));
        assert!(identifiers.contains(&"loc".to_string()));
        assert!(identifiers.contains(&"stars".to_string()));
    }

    #[test]
    fn test_two_conditions_in_func() {
        let source = r#"func Add(comp)
  ?loc.loop
    x = 1
  ?comp
    return
  x = 2"#;

        let result = parse_source(source);
        println!("Parse result: {:#?}", result);
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        if let Ok(program) = result {
            assert_eq!(program.statements.len(), 1);
            match &program.statements[0] {
                Statement::FunctionDefinition { body, .. } => {
                    // Should have 3 statements: first condition, second condition, assignment
                    assert!(
                        body.len() >= 2,
                        "Expected at least 2 statements in function body, got {}",
                        body.len()
                    );
                }
                _ => panic!("Expected function definition"),
            }
        }
    }

    #[test]
    fn test_two_conditions_in_func_crlf() {
        // Test with Windows line endings (CRLF)
        let source =
            "func Add(comp)\r\n  ?loc.loop\r\n    x = 1\r\n  ?comp\r\n    return\r\n  x = 2";

        let result = parse_source(source);
        println!("Parse result with CRLF: {:#?}", result);
        assert!(
            result.is_ok(),
            "Failed to parse with CRLF: {:?}",
            result.err()
        );

        if let Ok(program) = result {
            assert_eq!(program.statements.len(), 1);
            match &program.statements[0] {
                Statement::FunctionDefinition { body, .. } => {
                    // Should have 3 statements: first condition, second condition, assignment
                    assert!(
                        body.len() >= 2,
                        "Expected at least 2 statements in function body, got {}",
                        body.len()
                    );
                }
                _ => panic!("Expected function definition"),
            }
        }
    }

    #[test]
    fn test_dragcontroller_fragment() {
        // Test actual fragment from DragController.txt (no empty line between conditions)
        let source = "func Add(comp)\r\n  ?!hasCleared & loc.loop\r\n    draggables.Clear()\r\n    hasCleared=true\r\n  ?draggables.Contains(comp)\r\n    return\r\n  draggables.Add(comp)";

        let result = parse_source(source);
        println!("Parse DragController fragment: {:#?}", result);
        if let Err(e) = &result {
            println!("Error: {}", e);
        }
        assert!(
            result.is_ok(),
            "Failed to parse DragController fragment: {:?}",
            result.err()
        );

        if let Ok(program) = result {
            assert_eq!(program.statements.len(), 1);
            match &program.statements[0] {
                Statement::FunctionDefinition { body, .. } => {
                    println!("Function body has {} statements", body.len());
                    for (i, stmt) in body.iter().enumerate() {
                        println!("Statement {}: {:?}", i, stmt);
                    }
                    // Should have 3 statements: first condition, second condition, function call
                    assert!(
                        body.len() >= 3,
                        "Expected at least 3 statements in function body, got {}",
                        body.len()
                    );
                }
                _ => panic!("Expected function definition"),
            }
        }
    }

    #[test]
    fn test_full_dragcontroller() {
        // Test the entire DragController.txt file
        use std::fs;
        let content = fs::read_to_string("../../test_scripts/UI/DragController.txt")
            .expect("Could not read DragController.txt");

        let result = parse_source(&content);
        if let Err(e) = &result {
            println!("Error parsing DragController.txt: {}", e);
            // Find position in source
            if let Some(pos) = e.find("Remaining:") {
                let remaining_start = &e[pos + 11..];
                let preview = remaining_start.chars().take(100).collect::<String>();
                println!("Remaining preview: {:?}", preview);
            }
        }
        assert!(
            result.is_ok(),
            "Failed to parse DragController.txt: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_visitor() {
        struct CountingVisitor {
            statement_count: usize,
            expression_count: usize,
        }

        impl Visitor for CountingVisitor {
            fn visit_statement(&mut self, statement: &Statement) {
                self.statement_count += 1;
                // Continue with default behavior
                match statement {
                    Statement::Condition {
                        condition,
                        then_block,
                        else_ifs,
                        else_block,
                        ..
                    } => {
                        self.visit_condition(condition, then_block, else_ifs, else_block);
                    }
                    Statement::Command { name, args, .. } => {
                        self.visit_command(name, args);
                    }
                    Statement::Assignment { target, value, .. } => {
                        self.visit_assignment(target, value);
                    }
                    Statement::Output { position, text, .. } => {
                        self.visit_output(position, text);
                    }
                    _ => {}
                }
            }

            fn visit_expression(&mut self, expression: &Expression) {
                self.expression_count += 1;
                // Continue with default recursive visiting
                match expression {
                    Expression::Property { object, .. } => {
                        self.visit_expression(object);
                    }
                    Expression::BinaryOp { left, right, .. } => {
                        self.visit_expression(left);
                        self.visit_expression(right);
                    }
                    _ => {}
                }
            }
        }

        let source = "?hp < 10\n  activate potion";
        let program = parse_source(source).unwrap();

        let mut visitor = CountingVisitor {
            statement_count: 0,
            expression_count: 0,
        };
        visitor.visit_program(&program);

        assert!(visitor.statement_count > 0);
        assert!(visitor.expression_count > 0);
    }

    #[test]
    fn test_utf8_bom_handling() {
        let source_with_bom = "\u{FEFF}equip sword";
        let result = parse_source(source_with_bom);
        assert!(result.is_ok());
    }
}
