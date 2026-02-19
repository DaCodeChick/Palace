//! Lexer for Iptscrae scripting language.
//!
//! The lexer tokenizes Iptscrae source code into a sequence of tokens.
//! It handles:
//! - Comments (# to end of line)
//! - String literals ("...")
//! - Integer literals
//! - Identifiers and keywords
//! - Operators and delimiters

use crate::iptscrae::token::{SourcePos, Token, TokenKind};

/// Lexer error types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LexError {
    UnterminatedString {
        line: usize,
        column: usize,
    },
    InvalidCharacter {
        ch: char,
        line: usize,
        column: usize,
    },
    InvalidNumber {
        text: String,
        line: usize,
        column: usize,
    },
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexError::UnterminatedString { line, column } => {
                write!(f, "Unterminated string at line {}, column {}", line, column)
            }
            LexError::InvalidCharacter { ch, line, column } => {
                write!(
                    f,
                    "Invalid character '{}' at line {}, column {}",
                    ch, line, column
                )
            }
            LexError::InvalidNumber { text, line, column } => {
                write!(
                    f,
                    "Invalid number '{}' at line {}, column {}",
                    text, line, column
                )
            }
        }
    }
}

impl std::error::Error for LexError {}

/// Lexer for Iptscrae source code
pub struct Lexer {
    source: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    /// Create a new lexer from source code
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    /// Tokenize the entire source code
    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token()?;
            let is_eof = matches!(token.kind, TokenKind::Eof);
            tokens.push(token);
            if is_eof {
                break;
            }
        }

        Ok(tokens)
    }

    /// Get the next token
    pub fn next_token(&mut self) -> Result<Token, LexError> {
        // Skip whitespace (except newlines)
        self.skip_whitespace();

        let pos = self.current_pos();

        // Check for EOF
        if self.is_eof() {
            return Ok(Token::new(TokenKind::Eof, pos));
        }

        let ch = self.current_char();

        // Comments
        if ch == '#' {
            return Ok(self.lex_comment());
        }

        // Newlines
        if ch == '\n' || ch == '\r' {
            self.advance();
            if ch == '\r' && self.current_char() == '\n' {
                self.advance();
            }
            self.line += 1;
            self.column = 1;
            return Ok(Token::new(TokenKind::Newline, pos));
        }

        // String literals
        if ch == '"' {
            return self.lex_string();
        }

        // Numbers
        if ch.is_ascii_digit() || (ch == '-' && self.peek().is_some_and(|c| c.is_ascii_digit())) {
            return self.lex_number();
        }

        // Identifiers and keywords
        if ch.is_alphabetic() || ch == '_' {
            return Ok(self.lex_identifier());
        }

        // Operators and delimiters
        let token = match ch {
            '+' => {
                self.advance();
                Token::new(TokenKind::Plus, pos)
            }
            '-' => {
                self.advance();
                Token::new(TokenKind::Minus, pos)
            }
            '*' => {
                self.advance();
                Token::new(TokenKind::Star, pos)
            }
            '/' => {
                self.advance();
                Token::new(TokenKind::Slash, pos)
            }
            '%' => {
                self.advance();
                Token::new(TokenKind::Percent, pos)
            }
            '&' => {
                self.advance();
                Token::new(TokenKind::Ampersand, pos)
            }
            '=' => {
                self.advance();
                Token::new(TokenKind::Equals, pos)
            }
            '!' => {
                self.advance();
                if self.current_char() == '=' {
                    self.advance();
                    Token::new(TokenKind::NotEquals, pos)
                } else {
                    return Err(LexError::InvalidCharacter {
                        ch: '!',
                        line: pos.line,
                        column: pos.column,
                    });
                }
            }
            '<' => {
                self.advance();
                if self.current_char() == '=' {
                    self.advance();
                    Token::new(TokenKind::LessEq, pos)
                } else {
                    Token::new(TokenKind::Less, pos)
                }
            }
            '>' => {
                self.advance();
                if self.current_char() == '=' {
                    self.advance();
                    Token::new(TokenKind::GreaterEq, pos)
                } else {
                    Token::new(TokenKind::Greater, pos)
                }
            }
            '{' => {
                self.advance();
                Token::new(TokenKind::LeftBrace, pos)
            }
            '}' => {
                self.advance();
                Token::new(TokenKind::RightBrace, pos)
            }
            '(' => {
                self.advance();
                Token::new(TokenKind::LeftParen, pos)
            }
            ')' => {
                self.advance();
                Token::new(TokenKind::RightParen, pos)
            }
            ',' => {
                self.advance();
                Token::new(TokenKind::Comma, pos)
            }
            _ => {
                return Err(LexError::InvalidCharacter {
                    ch,
                    line: pos.line,
                    column: pos.column,
                });
            }
        };

        Ok(token)
    }

    /// Lex a comment
    fn lex_comment(&mut self) -> Token {
        let pos = self.current_pos();
        self.advance(); // Skip '#'

        let mut comment = String::new();
        while !self.is_eof() && self.current_char() != '\n' && self.current_char() != '\r' {
            comment.push(self.current_char());
            self.advance();
        }

        Token::new(TokenKind::Comment(comment), pos)
    }

    /// Lex a string literal
    fn lex_string(&mut self) -> Result<Token, LexError> {
        let pos = self.current_pos();
        self.advance(); // Skip opening quote

        let mut string = String::new();
        while !self.is_eof() && self.current_char() != '"' {
            let ch = self.current_char();
            if ch == '\\' {
                self.advance();
                if self.is_eof() {
                    return Err(LexError::UnterminatedString {
                        line: pos.line,
                        column: pos.column,
                    });
                }
                // Handle escape sequences
                let escaped = match self.current_char() {
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    '\\' => '\\',
                    '"' => '"',
                    c => c, // Unknown escape, just use the character
                };
                string.push(escaped);
                self.advance();
            } else {
                string.push(ch);
                self.advance();
            }
        }

        if self.is_eof() {
            return Err(LexError::UnterminatedString {
                line: pos.line,
                column: pos.column,
            });
        }

        self.advance(); // Skip closing quote
        Ok(Token::new(TokenKind::String(string), pos))
    }

    /// Lex a number (integer only)
    fn lex_number(&mut self) -> Result<Token, LexError> {
        let pos = self.current_pos();
        let mut number = String::new();

        // Handle negative sign
        if self.current_char() == '-' {
            number.push('-');
            self.advance();
        }

        // Collect digits
        while !self.is_eof() && self.current_char().is_ascii_digit() {
            number.push(self.current_char());
            self.advance();
        }

        // Parse the number
        match number.parse::<i32>() {
            Ok(n) => Ok(Token::new(TokenKind::Integer(n), pos)),
            Err(_) => Err(LexError::InvalidNumber {
                text: number,
                line: pos.line,
                column: pos.column,
            }),
        }
    }

    /// Lex an identifier or keyword
    fn lex_identifier(&mut self) -> Token {
        let pos = self.current_pos();
        let mut ident = String::new();

        while !self.is_eof() {
            let ch = self.current_char();
            if ch.is_alphanumeric() || ch == '_' {
                ident.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        let kind = TokenKind::from_ident(&ident);
        Token::new(kind, pos)
    }

    /// Skip whitespace characters (but not newlines)
    fn skip_whitespace(&mut self) {
        while !self.is_eof() {
            let ch = self.current_char();
            if ch == ' ' || ch == '\t' {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Get current character
    fn current_char(&self) -> char {
        if self.is_eof() {
            '\0'
        } else {
            self.source[self.position]
        }
    }

    /// Peek at next character
    fn peek(&self) -> Option<char> {
        if self.position + 1 < self.source.len() {
            Some(self.source[self.position + 1])
        } else {
            None
        }
    }

    /// Advance to next character
    fn advance(&mut self) {
        if !self.is_eof() {
            self.position += 1;
            self.column += 1;
        }
    }

    /// Check if at end of file
    const fn is_eof(&self) -> bool {
        self.position >= self.source.len()
    }

    /// Get current source position
    const fn current_pos(&self) -> SourcePos {
        SourcePos::new(self.line, self.column)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex_integers() {
        let mut lexer = Lexer::new("42 -17 0");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Integer(42));
        assert_eq!(tokens[1].kind, TokenKind::Integer(-17));
        assert_eq!(tokens[2].kind, TokenKind::Integer(0));
    }

    #[test]
    fn test_lex_strings() {
        let mut lexer = Lexer::new(r#""hello" "world" "test\"quote""#);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::String("hello".to_string()));
        assert_eq!(tokens[1].kind, TokenKind::String("world".to_string()));
        assert_eq!(tokens[2].kind, TokenKind::String("test\"quote".to_string()));
    }

    #[test]
    fn test_lex_identifiers() {
        let mut lexer = Lexer::new("foo bar_baz test123");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Ident("foo".to_string()));
        assert_eq!(tokens[1].kind, TokenKind::Ident("bar_baz".to_string()));
        assert_eq!(tokens[2].kind, TokenKind::Ident("test123".to_string()));
    }

    #[test]
    fn test_lex_keywords() {
        let mut lexer = Lexer::new("ON IF ELSE WHILE DO BREAK");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::On);
        assert_eq!(tokens[1].kind, TokenKind::If);
        assert_eq!(tokens[2].kind, TokenKind::Else);
        assert_eq!(tokens[3].kind, TokenKind::While);
        assert_eq!(tokens[4].kind, TokenKind::Do);
        assert_eq!(tokens[5].kind, TokenKind::Break);
    }

    #[test]
    fn test_lex_operators() {
        let mut lexer = Lexer::new("+ - * / % & = != < > <= >=");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Plus);
        assert_eq!(tokens[1].kind, TokenKind::Minus);
        assert_eq!(tokens[2].kind, TokenKind::Star);
        assert_eq!(tokens[3].kind, TokenKind::Slash);
        assert_eq!(tokens[4].kind, TokenKind::Percent);
        assert_eq!(tokens[5].kind, TokenKind::Ampersand);
        assert_eq!(tokens[6].kind, TokenKind::Equals);
        assert_eq!(tokens[7].kind, TokenKind::NotEquals);
        assert_eq!(tokens[8].kind, TokenKind::Less);
        assert_eq!(tokens[9].kind, TokenKind::Greater);
        assert_eq!(tokens[10].kind, TokenKind::LessEq);
        assert_eq!(tokens[11].kind, TokenKind::GreaterEq);
    }

    #[test]
    fn test_lex_delimiters() {
        let mut lexer = Lexer::new("{ } ( )");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::LeftBrace);
        assert_eq!(tokens[1].kind, TokenKind::RightBrace);
        assert_eq!(tokens[2].kind, TokenKind::LeftParen);
        assert_eq!(tokens[3].kind, TokenKind::RightParen);
    }

    #[test]
    fn test_lex_comments() {
        let mut lexer = Lexer::new("# This is a comment\n42");
        let tokens = lexer.tokenize().unwrap();

        assert!(matches!(tokens[0].kind, TokenKind::Comment(_)));
        assert_eq!(tokens[1].kind, TokenKind::Newline);
        assert_eq!(tokens[2].kind, TokenKind::Integer(42));
    }

    #[test]
    fn test_lex_script_example() {
        let source = r#"
            ON ENTER {
                "Welcome!" SAY
            }
        "#;
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();

        // Should contain ON, ENTER, {, String, Ident(SAY), }, EOF
        let kinds: Vec<_> = tokens
            .iter()
            .filter(|t| !matches!(t.kind, TokenKind::Newline))
            .map(|t| &t.kind)
            .collect();

        assert!(matches!(kinds[0], TokenKind::On));
        assert!(matches!(kinds[1], TokenKind::Ident(_)));
        assert!(matches!(kinds[2], TokenKind::LeftBrace));
        assert!(matches!(kinds[3], TokenKind::String(_)));
        assert!(matches!(kinds[4], TokenKind::Ident(_)));
        assert!(matches!(kinds[5], TokenKind::RightBrace));
        assert!(matches!(kinds[6], TokenKind::Eof));
    }

    #[test]
    fn test_unterminated_string() {
        let mut lexer = Lexer::new(r#""unterminated"#);
        let result = lexer.tokenize();
        assert!(matches!(result, Err(LexError::UnterminatedString { .. })));
    }

    #[test]
    fn test_invalid_character() {
        let mut lexer = Lexer::new("@invalid");
        let result = lexer.tokenize();
        assert!(matches!(result, Err(LexError::InvalidCharacter { .. })));
    }
}
