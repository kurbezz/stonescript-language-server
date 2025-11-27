//! Abstract Syntax Tree (AST) types for StoneScript

use std::fmt;

/// Position in source code (line and column, 0-indexed)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

/// Span representing a range in source code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl Span {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    pub fn contains_position(&self, pos: Position) -> bool {
        (pos.line > self.start.line
            || (pos.line == self.start.line && pos.column >= self.start.column))
            && (pos.line < self.end.line
                || (pos.line == self.end.line && pos.column <= self.end.column))
    }
}

/// Represents a complete StoneScript program
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
    pub span: Span,
}

/// A single statement in StoneScript
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// Conditional statement: ?condition
    Condition {
        condition: Expression,
        then_block: Vec<Statement>,
        else_ifs: Vec<ElseIf>,
        else_block: Option<Vec<Statement>>,
        span: Span,
    },
    /// Command statement (e.g., equip, loadout, activate)
    Command {
        name: String,
        args: Vec<Expression>,
        span: Span,
    },
    /// Variable declaration or assignment
    Assignment {
        target: Expression,
        op: AssignmentOperator,
        value: Expression,
        span: Span,
    },
    /// Output statement (>)
    Output {
        position: Option<(Expression, Expression)>,
        text: Expression,
        span: Span,
    },
    /// Expression used as statement (e.g., function call)
    ExpressionStatement { expression: Expression, span: Span },
    /// Function definition
    FunctionDefinition {
        name: String,
        params: Vec<String>,
        body: Vec<Statement>,
        span: Span,
    },
    /// Return statement
    Return {
        value: Option<Expression>,
        span: Span,
    },
    /// For loop with range
    For {
        variable: String,
        range: (Expression, Expression),
        body: Vec<Statement>,
        span: Span,
    },
    /// For-in loop (iterate over collection)
    ForIn {
        variable: String,
        collection: Expression,
        body: Vec<Statement>,
        span: Span,
    },
    /// While loop (adding for completeness, though not seen in Snake.txt yet)
    While {
        condition: Expression,
        body: Vec<Statement>,
        span: Span,
    },
    /// Import statement
    Import {
        path: String, // e.g., "Cosmetics/TrainAdventure/Main"
        span: Span,
    },
    /// Comment (// or /* */)
    Comment(String, Span),
    /// Empty line
    Empty,
}

impl Statement {
    pub fn span(&self) -> Option<Span> {
        match self {
            Statement::Condition { span, .. } => Some(*span),
            Statement::Command { span, .. } => Some(*span),
            Statement::Assignment { span, .. } => Some(*span),
            Statement::Output { span, .. } => Some(*span),
            Statement::ExpressionStatement { span, .. } => Some(*span),
            Statement::FunctionDefinition { span, .. } => Some(*span),
            Statement::Return { span, .. } => Some(*span),
            Statement::For { span, .. } => Some(*span),
            Statement::ForIn { span, .. } => Some(*span),
            Statement::While { span, .. } => Some(*span),
            Statement::Import { span, .. } => Some(*span),
            Statement::Comment(_, span) => Some(*span),
            Statement::Empty => None,
        }
    }
}

/// Else-if branch
#[derive(Debug, Clone, PartialEq)]
pub struct ElseIf {
    pub condition: Expression,
    pub block: Vec<Statement>,
    pub span: Span,
}

/// Expression types
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// Integer literal
    Integer(i64, Span),
    /// Float literal
    Float(f64, Span),
    /// Boolean literal
    Boolean(bool, Span),
    /// String literal
    String(String, Span),
    /// Identifier (variable or property)
    Identifier(String, Span),
    /// Property access (e.g., loc.stars, foe.hp)
    Property {
        object: Box<Expression>,
        property: String,
        span: Span,
    },
    /// Function call (e.g., foe.GetCount(46))
    FunctionCall {
        function: Box<Expression>,
        args: Vec<Expression>,
        span: Span,
    },
    /// Binary operation
    BinaryOp {
        left: Box<Expression>,
        op: BinaryOperator,
        right: Box<Expression>,
        span: Span,
    },
    /// Unary operation
    UnaryOp {
        op: UnaryOperator,
        operand: Box<Expression>,
        span: Span,
    },
    /// Interpolated string with `` backticks
    Interpolation(Vec<InterpolationPart>, Span),
    /// Object instantiation: new path
    New {
        path: String, // e.g., "Games/Fishing/FishingGame"
        span: Span,
    },
    /// Array literal (e.g., [], [1, 2, 3])
    Array {
        elements: Vec<Expression>,
        span: Span,
    },
    /// Index access (e.g., arr[0], obj.prop[index])
    IndexAccess {
        object: Box<Expression>,
        index: Box<Expression>,
        span: Span,
    },
}

impl Expression {
    pub fn span(&self) -> Span {
        match self {
            Expression::Integer(_, span) => *span,
            Expression::Float(_, span) => *span,
            Expression::Boolean(_, span) => *span,
            Expression::String(_, span) => *span,
            Expression::Identifier(_, span) => *span,
            Expression::Property { span, .. } => *span,
            Expression::FunctionCall { span, .. } => *span,
            Expression::BinaryOp { span, .. } => *span,
            Expression::UnaryOp { span, .. } => *span,
            Expression::Interpolation(_, span) => *span,
            Expression::New { span, .. } => *span,
            Expression::Array { span, .. } => *span,
            Expression::IndexAccess { span, .. } => *span,
        }
    }
}

/// Parts of string interpolation
#[derive(Debug, Clone, PartialEq)]
pub enum InterpolationPart {
    Text(String, Span),
    Expression(Box<Expression>),
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    // Comparison
    Equal,        // =
    NotEqual,     // !=
    Less,         // <
    LessEqual,    // <=
    Greater,      // >
    GreaterEqual, // >=

    // Logical
    And, // &
    Or,  // |

    // Arithmetic
    Add,      // +
    Subtract, // -
    Multiply, // *
    Divide,   // /
    Modulo,   // %
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    Not,       // !
    Negate,    // -
    Increment, // ++
    Decrement, // --
}

/// Assignment operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignmentOperator {
    Assign,         // =
    AddAssign,      // +=
    SubtractAssign, // -=
    MultiplyAssign, // *=
    DivideAssign,   // /=
    ModuloAssign,   // %=
}

impl fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOperator::Equal => write!(f, "="),
            BinaryOperator::NotEqual => write!(f, "!="),
            BinaryOperator::Less => write!(f, "<"),
            BinaryOperator::LessEqual => write!(f, "<="),
            BinaryOperator::Greater => write!(f, ">"),
            BinaryOperator::GreaterEqual => write!(f, ">="),
            BinaryOperator::And => write!(f, "&"),
            BinaryOperator::Or => write!(f, "|"),
            BinaryOperator::Add => write!(f, "+"),
            BinaryOperator::Subtract => write!(f, "-"),
            BinaryOperator::Multiply => write!(f, "*"),
            BinaryOperator::Divide => write!(f, "/"),
            BinaryOperator::Modulo => write!(f, "%"),
        }
    }
}

impl fmt::Display for UnaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOperator::Not => write!(f, "!"),
            UnaryOperator::Negate => write!(f, "-"),
            UnaryOperator::Increment => write!(f, "++"),
            UnaryOperator::Decrement => write!(f, "--"),
        }
    }
}
