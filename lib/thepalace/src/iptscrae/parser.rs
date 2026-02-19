//! Parser for Iptscrae scripting language.
//!
//! The parser converts a sequence of tokens into an Abstract Syntax Tree (AST).
//! Iptscrae is a stack-based language where most expressions push values onto
//! the stack, and operations consume values from the stack.

use crate::iptscrae::ast::{BinOp, Block, EventHandler, Expr, Script, Statement};
use crate::iptscrae::events::EventType;
use crate::iptscrae::token::{SourcePos, Token, TokenKind};
use crate::iptscrae::value::Value;

/// Parser error types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    UnexpectedToken {
        expected: String,
        found: String,
        pos: SourcePos,
    },
    UnexpectedEof {
        expected: String,
    },
    InvalidEventName {
        name: String,
        pos: SourcePos,
    },
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedToken {
                expected,
                found,
                pos,
            } => {
                write!(
                    f,
                    "Expected {} but found {} at line {}, column {}",
                    expected, found, pos.line, pos.column
                )
            }
            ParseError::UnexpectedEof { expected } => {
                write!(f, "Unexpected end of file, expected {}", expected)
            }
            ParseError::InvalidEventName { name, pos } => {
                write!(
                    f,
                    "Invalid event name '{}' at line {}, column {}",
                    name, pos.line, pos.column
                )
            }
        }
    }
}

impl std::error::Error for ParseError {}

/// Parser for Iptscrae source code
pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    /// Create a new parser from tokens
    pub const fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    /// Parse tokens into a script
    pub fn parse(&mut self) -> Result<Script, ParseError> {
        let mut handlers = Vec::new();

        // Skip initial newlines
        self.skip_newlines();

        while !self.is_at_end() {
            // Skip comments and newlines
            if self.skip_ignorable() {
                continue;
            }

            // Parse event handler
            if self.check(&TokenKind::On) {
                handlers.push(self.parse_event_handler()?);
            } else if self.is_at_end() {
                break;
            } else {
                let tok = self.current();
                return Err(ParseError::UnexpectedToken {
                    expected: "ON or end of file".to_string(),
                    found: self.token_description(&tok.kind),
                    pos: tok.pos,
                });
            }

            self.skip_newlines();
        }

        Ok(Script::new(handlers))
    }

    /// Parse an event handler: ON eventname { block }
    fn parse_event_handler(&mut self) -> Result<EventHandler, ParseError> {
        let pos = self.current().pos;
        self.consume(&TokenKind::On, "ON")?;

        // Parse event name
        let event_name = if let TokenKind::Ident(name) = &self.current().kind {
            name.clone()
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: "event name".to_string(),
                found: self.token_description(&self.current().kind),
                pos: self.current().pos,
            });
        };
        self.advance();

        // Convert event name to EventType
        let event =
            EventType::from_name(&event_name).ok_or_else(|| ParseError::InvalidEventName {
                name: event_name.clone(),
                pos,
            })?;

        self.skip_newlines();

        // Parse body block
        let body = self.parse_block()?;

        Ok(EventHandler::new(event, body, pos))
    }

    /// Parse a block: { statements }
    fn parse_block(&mut self) -> Result<Block, ParseError> {
        self.consume(&TokenKind::LeftBrace, "{")?;
        self.skip_newlines();

        let mut statements = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            self.skip_ignorable();

            if self.check(&TokenKind::RightBrace) {
                break;
            }

            statements.push(self.parse_statement()?);
            self.skip_newlines();
        }

        self.consume(&TokenKind::RightBrace, "}")?;
        Ok(Block::new(statements))
    }

    /// Parse a statement
    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        let pos = self.current().pos;

        // IF statement
        if self.check(&TokenKind::If) {
            return self.parse_if_statement();
        }

        // WHILE statement
        if self.check(&TokenKind::While) {
            return self.parse_while_statement();
        }

        // BREAK statement
        if self.check(&TokenKind::Break) {
            self.advance();
            return Ok(Statement::Break { pos });
        }

        // Otherwise, it's an expression or assignment
        // In Iptscrae, we parse the expression first, then check for assignment
        let expr = self.parse_expression()?;

        // Check for assignment (= after identifier)
        // In stack-based Iptscrae: "value name =" assigns value to name
        if self.check(&TokenKind::Equals) {
            self.advance();

            // The expression should have pushed the variable name
            // For now, we'll handle this in a simplified way:
            // Expect the pattern: expr ident =
            if let Expr::Variable { name, .. } = expr {
                return Ok(Statement::Assign { name, pos });
            } else {
                // In full Iptscrae, any expression can precede =
                // but we need the variable name on the stack
                // For now, we'll return it as an expr statement
                // and handle assignment more carefully later
                return Ok(Statement::Expr(expr));
            }
        }

        Ok(Statement::Expr(expr))
    }

    /// Parse an IF statement
    fn parse_if_statement(&mut self) -> Result<Statement, ParseError> {
        let pos = self.current().pos;
        self.consume(&TokenKind::If, "IF")?;

        self.skip_newlines();

        // Parse then block
        let then_block = self.parse_block()?;

        self.skip_newlines();

        // Parse optional else block
        let else_block = if self.check(&TokenKind::Else) {
            self.advance();
            self.skip_newlines();
            Some(self.parse_block()?)
        } else {
            None
        };

        // In Iptscrae, the condition is evaluated before IF
        // So we create an empty condition block
        // The actual condition value should be on the stack
        let condition = Block::new(vec![]);

        Ok(Statement::If {
            condition,
            then_block,
            else_block,
            pos,
        })
    }

    /// Parse a WHILE statement
    fn parse_while_statement(&mut self) -> Result<Statement, ParseError> {
        let pos = self.current().pos;
        self.consume(&TokenKind::While, "WHILE")?;

        self.skip_newlines();

        // Parse body block
        let body = self.parse_block()?;

        // In Iptscrae, the condition is evaluated before WHILE
        // Similar to IF, we use an empty condition block
        let condition = Block::new(vec![]);

        Ok(Statement::While {
            condition,
            body,
            pos,
        })
    }

    /// Parse an expression
    fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        self.parse_primary()
    }

    /// Parse a primary expression
    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        let pos = self.current().pos;

        match &self.current().kind {
            // Integer literal
            TokenKind::Integer(n) => {
                let value = *n;
                self.advance();
                Ok(Expr::Literal {
                    value: Value::Integer(value),
                    pos,
                })
            }

            // String literal
            TokenKind::String(s) => {
                let value = s.clone();
                self.advance();
                Ok(Expr::Literal {
                    value: Value::String(value),
                    pos,
                })
            }

            // Identifier (variable or function call)
            TokenKind::Ident(name) => {
                let name = name.clone();
                self.advance();

                // In Iptscrae, all identifiers can be either variables or function calls
                // We distinguish at runtime based on whether they're defined
                // For now, we'll treat uppercase names as calls and others as variables
                if name
                    .chars()
                    .all(|c| c.is_uppercase() || c == '_' || c.is_numeric())
                {
                    Ok(Expr::Call { name, pos })
                } else {
                    Ok(Expr::Variable { name, pos })
                }
            }

            // Operators (create BinOp or UnaryOp expressions)
            TokenKind::Plus => {
                self.advance();
                Ok(Expr::BinOp {
                    op: BinOp::Add,
                    pos,
                })
            }
            TokenKind::Minus => {
                self.advance();
                // Could be binary subtract or unary negate
                // For simplicity, treat as binary
                Ok(Expr::BinOp {
                    op: BinOp::Sub,
                    pos,
                })
            }
            TokenKind::Star => {
                self.advance();
                Ok(Expr::BinOp {
                    op: BinOp::Mul,
                    pos,
                })
            }
            TokenKind::Slash => {
                self.advance();
                Ok(Expr::BinOp {
                    op: BinOp::Div,
                    pos,
                })
            }
            TokenKind::Percent => {
                self.advance();
                Ok(Expr::BinOp {
                    op: BinOp::Mod,
                    pos,
                })
            }
            TokenKind::Ampersand => {
                self.advance();
                Ok(Expr::BinOp {
                    op: BinOp::Concat,
                    pos,
                })
            }
            TokenKind::NotEquals => {
                self.advance();
                Ok(Expr::BinOp {
                    op: BinOp::NotEq,
                    pos,
                })
            }
            TokenKind::Less => {
                self.advance();
                Ok(Expr::BinOp {
                    op: BinOp::Less,
                    pos,
                })
            }
            TokenKind::Greater => {
                self.advance();
                Ok(Expr::BinOp {
                    op: BinOp::Greater,
                    pos,
                })
            }
            TokenKind::LessEq => {
                self.advance();
                Ok(Expr::BinOp {
                    op: BinOp::LessEq,
                    pos,
                })
            }
            TokenKind::GreaterEq => {
                self.advance();
                Ok(Expr::BinOp {
                    op: BinOp::GreaterEq,
                    pos,
                })
            }

            // Block expression
            TokenKind::LeftBrace => {
                let block = self.parse_block()?;
                Ok(Expr::Block(block))
            }

            _ => Err(ParseError::UnexpectedToken {
                expected: "expression".to_string(),
                found: self.token_description(&self.current().kind),
                pos,
            }),
        }
    }

    /// Check if current token matches kind
    fn check(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() {
            return false;
        }
        std::mem::discriminant(&self.current().kind) == std::mem::discriminant(kind)
    }

    /// Consume a token of the expected kind
    fn consume(&mut self, kind: &TokenKind, expected: &str) -> Result<(), ParseError> {
        if self.check(kind) {
            self.advance();
            Ok(())
        } else if self.is_at_end() {
            Err(ParseError::UnexpectedEof {
                expected: expected.to_string(),
            })
        } else {
            Err(ParseError::UnexpectedToken {
                expected: expected.to_string(),
                found: self.token_description(&self.current().kind),
                pos: self.current().pos,
            })
        }
    }

    /// Get current token
    fn current(&self) -> &Token {
        &self.tokens[self.position]
    }

    /// Advance to next token
    fn advance(&mut self) {
        if !self.is_at_end() {
            self.position += 1;
        }
    }

    /// Check if at end of tokens
    fn is_at_end(&self) -> bool {
        self.position >= self.tokens.len()
            || matches!(self.tokens[self.position].kind, TokenKind::Eof)
    }

    /// Skip newlines
    fn skip_newlines(&mut self) {
        while !self.is_at_end() && matches!(self.current().kind, TokenKind::Newline) {
            self.advance();
        }
    }

    /// Skip ignorable tokens (newlines and comments)
    fn skip_ignorable(&mut self) -> bool {
        let mut skipped = false;
        while !self.is_at_end() {
            match &self.current().kind {
                TokenKind::Newline | TokenKind::Comment(_) => {
                    self.advance();
                    skipped = true;
                }
                _ => break,
            }
        }
        skipped
    }

    /// Get a description of a token for error messages
    fn token_description(&self, kind: &TokenKind) -> String {
        match kind {
            TokenKind::Integer(n) => format!("integer {}", n),
            TokenKind::String(s) => format!("string \"{}\"", s),
            TokenKind::Ident(name) => format!("identifier '{}'", name),
            TokenKind::On => "ON".to_string(),
            TokenKind::If => "IF".to_string(),
            TokenKind::Else => "ELSE".to_string(),
            TokenKind::While => "WHILE".to_string(),
            TokenKind::Do => "DO".to_string(),
            TokenKind::Break => "BREAK".to_string(),
            #[cfg(feature = "room-script")]
            TokenKind::Room => "ROOM".to_string(),
            #[cfg(feature = "room-script")]
            TokenKind::EndRoom => "ENDROOM".to_string(),
            #[cfg(feature = "room-script")]
            TokenKind::Door => "DOOR".to_string(),
            #[cfg(feature = "room-script")]
            TokenKind::EndDoor => "ENDDOOR".to_string(),
            #[cfg(feature = "room-script")]
            TokenKind::Spot => "SPOT".to_string(),
            #[cfg(feature = "room-script")]
            TokenKind::EndSpot => "ENDSPOT".to_string(),
            #[cfg(feature = "room-script")]
            TokenKind::Script => "SCRIPT".to_string(),
            #[cfg(feature = "room-script")]
            TokenKind::EndScript => "ENDSCRIPT".to_string(),
            #[cfg(feature = "room-script")]
            TokenKind::Id => "ID".to_string(),
            #[cfg(feature = "room-script")]
            TokenKind::Name => "NAME".to_string(),
            #[cfg(feature = "room-script")]
            TokenKind::Pict => "PICT".to_string(),
            #[cfg(feature = "room-script")]
            TokenKind::Artist => "ARTIST".to_string(),
            #[cfg(feature = "room-script")]
            TokenKind::Dest => "DEST".to_string(),
            #[cfg(feature = "room-script")]
            TokenKind::Outline => "OUTLINE".to_string(),
            #[cfg(feature = "room-script")]
            TokenKind::Picts => "PICTS".to_string(),
            #[cfg(feature = "room-script")]
            TokenKind::EndPicts => "ENDPICTS".to_string(),
            #[cfg(feature = "room-script")]
            TokenKind::Picture => "PICTURE".to_string(),
            #[cfg(feature = "room-script")]
            TokenKind::EndPicture => "ENDPICTURE".to_string(),
            #[cfg(feature = "room-script")]
            TokenKind::TransColor => "TRANSCOLOR".to_string(),
            #[cfg(feature = "room-script")]
            TokenKind::Private => "PRIVATE".to_string(),
            #[cfg(feature = "room-script")]
            TokenKind::NoPainting => "NOPAINTING".to_string(),
            #[cfg(feature = "room-script")]
            TokenKind::NoCyborgs => "NOCYBORGS".to_string(),
            #[cfg(feature = "room-script")]
            TokenKind::Hidden => "HIDDEN".to_string(),
            #[cfg(feature = "room-script")]
            TokenKind::NoGuests => "NOGUESTS".to_string(),
            TokenKind::Plus => "+".to_string(),
            TokenKind::Minus => "-".to_string(),
            TokenKind::Star => "*".to_string(),
            TokenKind::Slash => "/".to_string(),
            TokenKind::Percent => "%".to_string(),
            TokenKind::Ampersand => "&".to_string(),
            TokenKind::Equals => "=".to_string(),
            TokenKind::NotEquals => "!=".to_string(),
            TokenKind::Less => "<".to_string(),
            TokenKind::Greater => ">".to_string(),
            TokenKind::LessEq => "<=".to_string(),
            TokenKind::GreaterEq => ">=".to_string(),
            TokenKind::LeftBrace => "{".to_string(),
            TokenKind::RightBrace => "}".to_string(),
            TokenKind::LeftParen => "(".to_string(),
            TokenKind::RightParen => ")".to_string(),
            TokenKind::Comma => ",".to_string(),
            TokenKind::Comment(_) => "comment".to_string(),
            TokenKind::Newline => "newline".to_string(),
            TokenKind::Eof => "end of file".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::iptscrae::lexer::Lexer;

    fn parse_source(source: &str) -> Result<Script, ParseError> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    #[test]
    fn test_parse_simple_handler() {
        let source = r#"
            ON ENTER {
                "Hello" SAY
            }
        "#;
        let script = parse_source(source).unwrap();
        assert_eq!(script.handlers.len(), 1);
        assert_eq!(script.handlers[0].event, EventType::Enter);
    }

    #[test]
    fn test_parse_multiple_handlers() {
        let source = r#"
            ON ENTER {
                "Entering" SAY
            }
            ON LEAVE {
                "Leaving" SAY
            }
        "#;
        let script = parse_source(source).unwrap();
        assert_eq!(script.handlers.len(), 2);
        assert_eq!(script.handlers[0].event, EventType::Enter);
        assert_eq!(script.handlers[1].event, EventType::Leave);
    }

    #[test]
    fn test_parse_literals() {
        let source = r#"
            ON SELECT {
                42
                "test"
            }
        "#;
        let script = parse_source(source).unwrap();
        assert_eq!(script.handlers.len(), 1);

        let statements = &script.handlers[0].body.statements;
        assert_eq!(statements.len(), 2);

        if let Statement::Expr(Expr::Literal {
            value: Value::Integer(n),
            ..
        }) = &statements[0]
        {
            assert_eq!(*n, 42);
        } else {
            panic!("Expected integer literal");
        }

        if let Statement::Expr(Expr::Literal {
            value: Value::String(s),
            ..
        }) = &statements[1]
        {
            assert_eq!(s, "test");
        } else {
            panic!("Expected string literal");
        }
    }

    #[test]
    fn test_parse_function_calls() {
        let source = r#"
            ON ENTER {
                WHONAME SAY
            }
        "#;
        let script = parse_source(source).unwrap();
        let statements = &script.handlers[0].body.statements;
        assert_eq!(statements.len(), 2);

        assert!(
            matches!(&statements[0], Statement::Expr(Expr::Call { name, .. }) if name == "WHONAME")
        );
        assert!(
            matches!(&statements[1], Statement::Expr(Expr::Call { name, .. }) if name == "SAY")
        );
    }

    #[test]
    fn test_parse_if_statement() {
        let source = r#"
            ON SELECT {
                count 10 < IF {
                    "Less than 10" SAY
                }
            }
        "#;
        let script = parse_source(source).unwrap();
        let statements = &script.handlers[0].body.statements;

        // Should have: count, 10, <, IF
        assert!(statements.len() >= 4);
        assert!(matches!(&statements[3], Statement::If { .. }));
    }

    #[test]
    fn test_parse_while_statement() {
        let source = r#"
            ON STARTUP {
                { count 10 < } WHILE {
                    count 1 + count =
                }
            }
        "#;
        let script = parse_source(source).unwrap();
        let statements = &script.handlers[0].body.statements;

        // Should have: block, WHILE
        assert!(statements.len() >= 2);
    }

    #[test]
    fn test_parse_invalid_event() {
        let source = r#"
            ON INVALIDEVENT {
                "test" SAY
            }
        "#;
        let result = parse_source(source);
        assert!(matches!(result, Err(ParseError::InvalidEventName { .. })));
    }

    #[test]
    fn test_parse_unterminated_block() {
        let source = r#"
            ON ENTER {
                "test" SAY
        "#;
        let result = parse_source(source);
        assert!(result.is_err());
    }
}
