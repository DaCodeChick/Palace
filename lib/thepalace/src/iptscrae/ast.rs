//! Abstract Syntax Tree for Iptscrae.
//!
//! The AST represents the structure of parsed Iptscrae code before execution.

use crate::iptscrae::events::EventType;
use crate::iptscrae::token::SourcePos;
use crate::iptscrae::value::Value;

/// Top-level script containing event handlers
#[derive(Debug, Clone, PartialEq)]
pub struct Script {
    pub handlers: Vec<EventHandler>,
}

impl Script {
    pub const fn new(handlers: Vec<EventHandler>) -> Self {
        Self { handlers }
    }
}

/// Event handler (ON eventname { statements })
#[derive(Debug, Clone, PartialEq)]
pub struct EventHandler {
    pub event: EventType,
    pub body: Block,
    pub pos: SourcePos,
}

impl EventHandler {
    pub const fn new(event: EventType, body: Block, pos: SourcePos) -> Self {
        Self { event, body, pos }
    }
}

/// Block of statements
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<Statement>,
}

impl Block {
    pub const fn new(statements: Vec<Statement>) -> Self {
        Self { statements }
    }
}

/// Statement
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// Expression statement (most common - push values, call functions)
    Expr(Expr),

    /// Variable assignment (value name =)
    Assign { name: String, pos: SourcePos },

    /// If statement (condition { then_block } [ELSE { else_block }])
    If {
        condition: Block,
        then_block: Block,
        else_block: Option<Block>,
        pos: SourcePos,
    },

    /// While loop (condition { body })
    While {
        condition: Block,
        body: Block,
        pos: SourcePos,
    },

    /// Break from loop
    Break { pos: SourcePos },
}

/// Expression
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Literal value
    Literal { value: Value, pos: SourcePos },

    /// Variable reference (pushes variable value onto stack)
    Variable { name: String, pos: SourcePos },

    /// Function call (identifier that calls a built-in function)
    Call { name: String, pos: SourcePos },

    /// Binary operation
    BinOp { op: BinOp, pos: SourcePos },

    /// Unary operation
    UnaryOp { op: UnaryOp, pos: SourcePos },

    /// Block expression (used for control flow conditions)
    Block(Block),
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    // Comparison
    Eq,
    NotEq,
    Less,
    Greater,
    LessEq,
    GreaterEq,

    // Logical
    And,
    Or,
    Xor,

    // String
    Concat, // &
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

impl BinOp {
    /// Get precedence level (higher = tighter binding)
    pub const fn precedence(self) -> u8 {
        match self {
            BinOp::Or | BinOp::Xor => 1,
            BinOp::And => 2,
            BinOp::Eq | BinOp::NotEq => 3,
            BinOp::Less | BinOp::Greater | BinOp::LessEq | BinOp::GreaterEq => 4,
            BinOp::Add | BinOp::Sub | BinOp::Concat => 5,
            BinOp::Mul | BinOp::Div | BinOp::Mod => 6,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_creation() {
        let handler = EventHandler::new(EventType::Enter, Block::new(vec![]), SourcePos::new(1, 1));
        let script = Script::new(vec![handler.clone()]);
        assert_eq!(script.handlers.len(), 1);
        assert_eq!(script.handlers[0], handler);
    }

    #[test]
    fn test_binop_precedence() {
        assert!(BinOp::Mul.precedence() > BinOp::Add.precedence());
        assert!(BinOp::Add.precedence() > BinOp::Eq.precedence());
        assert!(BinOp::Eq.precedence() > BinOp::And.precedence());
        assert!(BinOp::And.precedence() > BinOp::Or.precedence());
    }

    #[test]
    fn test_expr_types() {
        let lit = Expr::Literal {
            value: Value::Integer(42),
            pos: SourcePos::new(1, 1),
        };
        assert!(matches!(lit, Expr::Literal { .. }));

        let var = Expr::Variable {
            name: "test".to_string(),
            pos: SourcePos::new(1, 1),
        };
        assert!(matches!(var, Expr::Variable { .. }));
    }
}
