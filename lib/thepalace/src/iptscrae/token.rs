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

    // Room script keywords (only available with room-script feature)
    #[cfg(feature = "room-script")]
    Room, // ROOM
    #[cfg(feature = "room-script")]
    EndRoom, // ENDROOM
    #[cfg(feature = "room-script")]
    Door, // DOOR
    #[cfg(feature = "room-script")]
    EndDoor, // ENDDOOR
    #[cfg(feature = "room-script")]
    Spot, // SPOT
    #[cfg(feature = "room-script")]
    EndSpot, // ENDSPOT
    #[cfg(feature = "room-script")]
    Script, // SCRIPT
    #[cfg(feature = "room-script")]
    EndScript, // ENDSCRIPT
    #[cfg(feature = "room-script")]
    Id, // ID
    #[cfg(feature = "room-script")]
    Name, // NAME
    #[cfg(feature = "room-script")]
    Pict, // PICT
    #[cfg(feature = "room-script")]
    Artist, // ARTIST
    #[cfg(feature = "room-script")]
    Dest, // DEST
    #[cfg(feature = "room-script")]
    Outline, // OUTLINE
    #[cfg(feature = "room-script")]
    Picts, // PICTS
    #[cfg(feature = "room-script")]
    EndPicts, // ENDPICTS
    #[cfg(feature = "room-script")]
    Picture, // PICTURE
    #[cfg(feature = "room-script")]
    EndPicture, // ENDPICTURE
    #[cfg(feature = "room-script")]
    TransColor, // TRANSCOLOR
    #[cfg(feature = "room-script")]
    Private, // PRIVATE
    #[cfg(feature = "room-script")]
    NoPainting, // NOPAINTING
    #[cfg(feature = "room-script")]
    NoCyborgs, // NOCYBORGS
    #[cfg(feature = "room-script")]
    Hidden, // HIDDEN
    #[cfg(feature = "room-script")]
    NoGuests, // NOGUESTS

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
    Comma,      // , (for OUTLINE and PICTS)

    // Special
    Comment(String), // # comment
    Newline,
    Eof,
}

impl TokenKind {
    /// Check if token is a keyword
    pub fn is_keyword(&self) -> bool {
        #[cfg(not(feature = "room-script"))]
        {
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
        #[cfg(feature = "room-script")]
        {
            matches!(
                self,
                TokenKind::On
                    | TokenKind::If
                    | TokenKind::Else
                    | TokenKind::While
                    | TokenKind::Do
                    | TokenKind::Break
                    | TokenKind::Room
                    | TokenKind::EndRoom
                    | TokenKind::Door
                    | TokenKind::EndDoor
                    | TokenKind::Spot
                    | TokenKind::EndSpot
                    | TokenKind::Script
                    | TokenKind::EndScript
                    | TokenKind::Id
                    | TokenKind::Name
                    | TokenKind::Pict
                    | TokenKind::Artist
                    | TokenKind::Dest
                    | TokenKind::Outline
                    | TokenKind::Picts
                    | TokenKind::EndPicts
                    | TokenKind::Picture
                    | TokenKind::EndPicture
                    | TokenKind::TransColor
                    | TokenKind::Private
                    | TokenKind::NoPainting
                    | TokenKind::NoCyborgs
                    | TokenKind::Hidden
                    | TokenKind::NoGuests
            )
        }
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
            #[cfg(feature = "room-script")]
            "ROOM" => TokenKind::Room,
            #[cfg(feature = "room-script")]
            "ENDROOM" => TokenKind::EndRoom,
            #[cfg(feature = "room-script")]
            "DOOR" => TokenKind::Door,
            #[cfg(feature = "room-script")]
            "ENDDOOR" => TokenKind::EndDoor,
            #[cfg(feature = "room-script")]
            "SPOT" => TokenKind::Spot,
            #[cfg(feature = "room-script")]
            "ENDSPOT" => TokenKind::EndSpot,
            #[cfg(feature = "room-script")]
            "SCRIPT" => TokenKind::Script,
            #[cfg(feature = "room-script")]
            "ENDSCRIPT" => TokenKind::EndScript,
            #[cfg(feature = "room-script")]
            "ID" => TokenKind::Id,
            #[cfg(feature = "room-script")]
            "NAME" => TokenKind::Name,
            #[cfg(feature = "room-script")]
            "PICT" => TokenKind::Pict,
            #[cfg(feature = "room-script")]
            "ARTIST" => TokenKind::Artist,
            #[cfg(feature = "room-script")]
            "DEST" => TokenKind::Dest,
            #[cfg(feature = "room-script")]
            "OUTLINE" => TokenKind::Outline,
            #[cfg(feature = "room-script")]
            "PICTS" => TokenKind::Picts,
            #[cfg(feature = "room-script")]
            "ENDPICTS" => TokenKind::EndPicts,
            #[cfg(feature = "room-script")]
            "PICTURE" => TokenKind::Picture,
            #[cfg(feature = "room-script")]
            "ENDPICTURE" => TokenKind::EndPicture,
            #[cfg(feature = "room-script")]
            "TRANSCOLOR" => TokenKind::TransColor,
            #[cfg(feature = "room-script")]
            "PRIVATE" => TokenKind::Private,
            #[cfg(feature = "room-script")]
            "NOPAINTING" => TokenKind::NoPainting,
            #[cfg(feature = "room-script")]
            "NOCYBORGS" => TokenKind::NoCyborgs,
            #[cfg(feature = "room-script")]
            "HIDDEN" => TokenKind::Hidden,
            #[cfg(feature = "room-script")]
            "NOGUESTS" => TokenKind::NoGuests,
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
