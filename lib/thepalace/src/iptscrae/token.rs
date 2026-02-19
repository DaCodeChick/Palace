//! Token types for Iptscrae lexer.
//!
//! This module defines all token types that can appear in Iptscrae source code.

/// Position in source code (line and column)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourcePos {
    pub line: usize,
    pub column: usize,
}

impl SourcePos {
    pub const fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

/// Token with position information
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub pos: SourcePos,
}

impl Token {
    pub const fn new(kind: TokenKind, pos: SourcePos) -> Self {
        Self { kind, pos }
    }
}

/// Token types
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    Integer(i32),
    String(String),

    // Identifiers (variables and function names)
    Ident(String),

    // Keywords
    On,    // ON
    If,    // IF
    Else,  // ELSE
    While, // WHILE
    Do,    // DO
    Break, // BREAK

    // Operators
    Plus,      // +
    Minus,     // -
    Star,      // *
    Slash,     // /
    Percent,   // % (MOD alternative)
    Ampersand, // & (string concatenation)
    Equals,    // =
    NotEquals, // !=
    Less,      // <
    Greater,   // >
    LessEq,    // <=
    GreaterEq, // >=

    // Delimiters
    LeftBrace,  // {
    RightBrace, // }
    LeftParen,  // (
    RightParen, // )

    // Special
    Comment(String), // # comment
    Newline,
    Eof,
}

impl TokenKind {
    /// Check if token is a keyword
    pub fn is_keyword(&self) -> bool {
        matches!(
            self,
            TokenKind::On
                | TokenKind::If
                | TokenKind::Else
                | TokenKind::While
                | TokenKind::Do
                | TokenKind::Break
        )
    }

    /// Try to parse identifier as keyword
    pub fn from_ident(ident: &str) -> Self {
        match ident.to_uppercase().as_str() {
            "ON" => TokenKind::On,
            "IF" => TokenKind::If,
            "ELSE" => TokenKind::Else,
            "WHILE" => TokenKind::While,
            "DO" => TokenKind::Do,
            "BREAK" => TokenKind::Break,
            _ => TokenKind::Ident(ident.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_detection() {
        assert!(TokenKind::On.is_keyword());
        assert!(TokenKind::If.is_keyword());
        assert!(!TokenKind::Ident("test".to_string()).is_keyword());
    }

    #[test]
    fn test_from_ident() {
        assert_eq!(TokenKind::from_ident("ON"), TokenKind::On);
        assert_eq!(TokenKind::from_ident("on"), TokenKind::On);
        assert_eq!(TokenKind::from_ident("IF"), TokenKind::If);
        assert_eq!(
            TokenKind::from_ident("test"),
            TokenKind::Ident("test".to_string())
        );
    }

    #[test]
    fn test_source_pos() {
        let pos = SourcePos::new(1, 5);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 5);
    }
}
