//! Room script parser for Palace server script files.
//!
//! This parser handles the meta-syntax for defining rooms, doors, and spots.

use crate::iptscrae::{
    DoorDecl, LexError, Lexer, ParseError, Parser, PictureDecl, RoomDecl, RoomFlags, Script,
    SourcePos, SpotDecl, StateDecl, Token, TokenKind,
};
use crate::Point;

/// Parser for room script files (e.g., Mansion.ipt).
pub struct RoomScriptParser {
    tokens: Vec<Token>,
    pos: usize,
}

impl RoomScriptParser {
    /// Create a new room script parser from source code.
    pub fn new(source: &str) -> Result<Self, ParseError> {
        let mut lexer = Lexer::new(source);
        let mut tokens = Vec::new();

        loop {
            let token = lexer.next_token().map_err(|e| {
                // Convert LexError to ParseError
                ParseError::UnexpectedToken {
                    expected: "valid token".to_string(),
                    found: format!("lexer error: {:?}", e),
                    pos: match e {
                        LexError::UnterminatedString { line, column } => SourcePos { line, column },
                        LexError::InvalidCharacter { line, column, .. } => {
                            SourcePos { line, column }
                        }
                        LexError::InvalidNumber { line, column, .. } => SourcePos { line, column },
                    },
                }
            })?;
            let is_eof = matches!(token.kind, TokenKind::Eof);
            tokens.push(token);
            if is_eof {
                break;
            }
        }

        Ok(Self { tokens, pos: 0 })
    }

    /// Parse multiple room declarations from a server script file.
    pub fn parse(&mut self) -> Result<Vec<RoomDecl>, ParseError> {
        let mut rooms = Vec::new();

        // Skip any leading newlines
        self.skip_newlines();

        while !self.is_at_end() {
            // Skip comments and newlines between rooms
            if matches!(
                self.current().kind,
                TokenKind::Comment(_) | TokenKind::Newline
            ) {
                self.advance();
                continue;
            }

            // Parse a room declaration
            if matches!(self.current().kind, TokenKind::Room) {
                rooms.push(self.parse_room()?);
            } else {
                return Err(self.error(format!(
                    "Expected ROOM keyword, found {}",
                    self.token_description(&self.current().kind)
                )));
            }

            self.skip_newlines();
        }

        Ok(rooms)
    }

    /// Parse a single ROOM ... ENDROOM block.
    fn parse_room(&mut self) -> Result<RoomDecl, ParseError> {
        self.expect(TokenKind::Room)?;
        self.skip_newlines();

        let mut id = None;
        let mut name = None;
        let mut pict = None;
        let mut artist = None;
        let password = None;
        let mut flags = RoomFlags::default();
        let mut pictures = Vec::new();
        let mut doors = Vec::new();
        let mut spots = Vec::new();

        // Parse room properties and nested elements
        while !self.is_at_end() && !matches!(self.current().kind, TokenKind::EndRoom) {
            self.skip_newlines();

            match &self.current().kind {
                TokenKind::Id => {
                    self.advance();
                    id = Some(self.parse_i16()?);
                    self.skip_newlines();
                }
                TokenKind::Name => {
                    self.advance();
                    name = Some(self.parse_string()?);
                    self.skip_newlines();
                }
                TokenKind::Pict => {
                    self.advance();
                    pict = Some(self.parse_string()?);
                    self.skip_newlines();
                }
                TokenKind::Artist => {
                    self.advance();
                    artist = Some(self.parse_string()?);
                    self.skip_newlines();
                }
                TokenKind::Private => {
                    self.advance();
                    flags.private = true;
                    self.skip_newlines();
                }
                TokenKind::NoPainting => {
                    self.advance();
                    flags.no_painting = true;
                    self.skip_newlines();
                }
                TokenKind::NoCyborgs => {
                    self.advance();
                    flags.no_cyborgs = true;
                    self.skip_newlines();
                }
                TokenKind::Hidden => {
                    self.advance();
                    flags.hidden = true;
                    self.skip_newlines();
                }
                TokenKind::NoGuests => {
                    self.advance();
                    flags.no_guests = true;
                    self.skip_newlines();
                }
                TokenKind::Picture => {
                    pictures.push(self.parse_picture()?);
                    self.skip_newlines();
                }
                TokenKind::Door => {
                    doors.push(self.parse_door()?);
                    self.skip_newlines();
                }
                TokenKind::Spot => {
                    spots.push(self.parse_spot()?);
                    self.skip_newlines();
                }
                TokenKind::Comment(_) | TokenKind::Newline => {
                    self.advance();
                }
                _ => {
                    return Err(self.error(format!(
                        "Unexpected token in room block: {}",
                        self.token_description(&self.current().kind)
                    )));
                }
            }
        }

        self.expect(TokenKind::EndRoom)?;

        let id = id.ok_or_else(|| self.error("Room must have an ID".to_string()))?;

        Ok(RoomDecl {
            id,
            name,
            pict,
            artist,
            password,
            flags,
            pictures,
            doors,
            spots,
        })
    }

    /// Parse a PICTURE ... ENDPICTURE block.
    fn parse_picture(&mut self) -> Result<PictureDecl, ParseError> {
        self.expect(TokenKind::Picture)?;
        self.skip_newlines();

        let mut id = None;
        let mut name = None;
        let mut trans_color = None;

        while !self.is_at_end() && !matches!(self.current().kind, TokenKind::EndPicture) {
            self.skip_newlines();

            match &self.current().kind {
                TokenKind::Id => {
                    self.advance();
                    id = Some(self.parse_i16()?);
                    self.skip_newlines();
                }
                TokenKind::Name => {
                    self.advance();
                    name = Some(self.parse_string()?);
                    self.skip_newlines();
                }
                TokenKind::TransColor => {
                    self.advance();
                    trans_color = Some(self.parse_i16()?);
                    self.skip_newlines();
                }
                TokenKind::Comment(_) | TokenKind::Newline => {
                    self.advance();
                }
                _ => {
                    return Err(self.error(format!(
                        "Unexpected token in PICTURE block: {}",
                        self.token_description(&self.current().kind)
                    )));
                }
            }
        }

        self.expect(TokenKind::EndPicture)?;

        let id = id.ok_or_else(|| self.error("PICTURE must have an ID".to_string()))?;
        let name = name.ok_or_else(|| self.error("PICTURE must have a NAME".to_string()))?;

        Ok(PictureDecl {
            id,
            name,
            trans_color,
        })
    }

    /// Parse a DOOR ... ENDDOOR block.
    fn parse_door(&mut self) -> Result<DoorDecl, ParseError> {
        self.expect(TokenKind::Door)?;
        self.skip_newlines();

        let mut id = None;
        let mut dest = None;
        let mut name = None;
        let mut outline = Vec::new();
        let mut picts = Vec::new();
        let mut script = None;

        while !self.is_at_end() && !matches!(self.current().kind, TokenKind::EndDoor) {
            self.skip_newlines();

            match &self.current().kind {
                TokenKind::Id => {
                    self.advance();
                    id = Some(self.parse_i16()?);
                    self.skip_newlines();
                }
                TokenKind::Dest => {
                    self.advance();
                    dest = Some(self.parse_i16()?);
                    self.skip_newlines();
                }
                TokenKind::Name => {
                    self.advance();
                    name = Some(self.parse_string()?);
                    self.skip_newlines();
                }
                TokenKind::Outline => {
                    self.advance();
                    outline = self.parse_outline()?;
                    self.skip_newlines();
                }
                TokenKind::Picts => {
                    picts = self.parse_picts()?;
                    self.skip_newlines();
                }
                TokenKind::Script => {
                    script = Some(self.parse_script_block()?);
                    self.skip_newlines();
                }
                TokenKind::Comment(_) | TokenKind::Newline => {
                    self.advance();
                }
                _ => {
                    return Err(self.error(format!(
                        "Unexpected token in DOOR block: {}",
                        self.token_description(&self.current().kind)
                    )));
                }
            }
        }

        self.expect(TokenKind::EndDoor)?;

        let id = id.ok_or_else(|| self.error("DOOR must have an ID".to_string()))?;
        let dest = dest.ok_or_else(|| self.error("DOOR must have a DEST".to_string()))?;

        Ok(DoorDecl {
            id,
            dest,
            name,
            outline,
            picts,
            script,
        })
    }

    /// Parse a SPOT ... ENDSPOT block.
    fn parse_spot(&mut self) -> Result<SpotDecl, ParseError> {
        self.expect(TokenKind::Spot)?;
        self.skip_newlines();

        let mut id = None;
        let mut name = None;
        let mut outline = Vec::new();
        let mut picts = Vec::new();
        let mut script = None;

        while !self.is_at_end() && !matches!(self.current().kind, TokenKind::EndSpot) {
            self.skip_newlines();

            match &self.current().kind {
                TokenKind::Id => {
                    self.advance();
                    id = Some(self.parse_i16()?);
                    self.skip_newlines();
                }
                TokenKind::Name => {
                    self.advance();
                    name = Some(self.parse_string()?);
                    self.skip_newlines();
                }
                TokenKind::Outline => {
                    self.advance();
                    outline = self.parse_outline()?;
                    self.skip_newlines();
                }
                TokenKind::Picts => {
                    picts = self.parse_picts()?;
                    self.skip_newlines();
                }
                TokenKind::Script => {
                    script = Some(self.parse_script_block()?);
                    self.skip_newlines();
                }
                TokenKind::Comment(_) | TokenKind::Newline => {
                    self.advance();
                }
                _ => {
                    return Err(self.error(format!(
                        "Unexpected token in SPOT block: {}",
                        self.token_description(&self.current().kind)
                    )));
                }
            }
        }

        self.expect(TokenKind::EndSpot)?;

        let id = id.ok_or_else(|| self.error("SPOT must have an ID".to_string()))?;

        Ok(SpotDecl {
            id,
            name,
            outline,
            picts,
            script,
        })
    }

    /// Parse OUTLINE x,y x,y x,y ...
    fn parse_outline(&mut self) -> Result<Vec<Point>, ParseError> {
        let mut points = Vec::new();

        // Parse coordinate pairs separated by whitespace
        loop {
            self.skip_newlines();

            // Check if we're at the end of the outline (keyword or end of section)
            if matches!(
                self.current().kind,
                TokenKind::EndRoom
                    | TokenKind::EndDoor
                    | TokenKind::EndSpot
                    | TokenKind::Door
                    | TokenKind::Spot
                    | TokenKind::Picture
                    | TokenKind::Picts
                    | TokenKind::Script
                    | TokenKind::Id
                    | TokenKind::Name
                    | TokenKind::Dest
                    | TokenKind::Pict
                    | TokenKind::Artist
                    | TokenKind::Private
                    | TokenKind::NoPainting
                    | TokenKind::NoCyborgs
                    | TokenKind::Hidden
                    | TokenKind::NoGuests
            ) {
                break;
            }

            // Parse x coordinate
            let h = self.parse_i16()?;

            // Expect comma
            self.expect(TokenKind::Comma)?;

            // Parse y coordinate
            let v = self.parse_i16()?;

            points.push(Point { h, v });
        }

        Ok(points)
    }

    /// Parse PICTS picID,xOffset,yOffset ... ENDPICTS
    fn parse_picts(&mut self) -> Result<Vec<StateDecl>, ParseError> {
        self.expect(TokenKind::Picts)?;
        self.skip_newlines();

        let mut states = Vec::new();

        while !self.is_at_end() && !matches!(self.current().kind, TokenKind::EndPicts) {
            self.skip_newlines();

            if matches!(
                self.current().kind,
                TokenKind::Comment(_) | TokenKind::Newline
            ) {
                self.advance();
                continue;
            }

            // Parse pic_id
            let pic_id = self.parse_i16()?;
            self.expect(TokenKind::Comma)?;

            // Parse x_offset
            let x_offset = self.parse_i16()?;
            self.expect(TokenKind::Comma)?;

            // Parse y_offset
            let y_offset = self.parse_i16()?;

            states.push(StateDecl {
                pic_id,
                x_offset,
                y_offset,
            });

            self.skip_newlines();
        }

        self.expect(TokenKind::EndPicts)?;

        Ok(states)
    }

    /// Parse SCRIPT ... ENDSCRIPT block using the regular iptscrae parser.
    fn parse_script_block(&mut self) -> Result<Script, ParseError> {
        self.expect(TokenKind::Script)?;
        self.skip_newlines();

        // Collect tokens until ENDSCRIPT
        let mut script_tokens = Vec::new();
        let mut depth = 1; // We're already inside SCRIPT

        while !self.is_at_end() && depth > 0 {
            let token = self.current().clone();

            match &token.kind {
                TokenKind::Script => depth += 1,
                TokenKind::EndScript => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }

            script_tokens.push(token);
            self.advance();
        }

        self.expect(TokenKind::EndScript)?;

        // Add EOF token for the parser
        if let Some(last_token) = script_tokens.last() {
            script_tokens.push(Token::new(TokenKind::Eof, last_token.pos));
        }

        // Parse the collected tokens as a regular iptscrae script
        let mut parser = Parser::new(script_tokens);
        parser.parse()
    }

    // Helper methods

    fn current(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn is_at_end(&self) -> bool {
        matches!(self.current().kind, TokenKind::Eof)
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            self.pos += 1;
        }
    }

    fn skip_newlines(&mut self) {
        while matches!(
            self.current().kind,
            TokenKind::Newline | TokenKind::Comment(_)
        ) {
            self.advance();
        }
    }

    fn expect(&mut self, kind: TokenKind) -> Result<(), ParseError> {
        if std::mem::discriminant(&self.current().kind) != std::mem::discriminant(&kind) {
            return Err(self.error(format!(
                "Expected {}, found {}",
                self.token_description(&kind),
                self.token_description(&self.current().kind)
            )));
        }
        self.advance();
        Ok(())
    }

    fn parse_i16(&mut self) -> Result<i16, ParseError> {
        match &self.current().kind {
            TokenKind::Integer(n) => {
                let value = *n;
                self.advance();
                if value < i16::MIN as i32 || value > i16::MAX as i32 {
                    Err(self.error(format!("Integer {} out of range for i16", value)))
                } else {
                    Ok(value as i16)
                }
            }
            TokenKind::Minus => {
                self.advance();
                match &self.current().kind {
                    TokenKind::Integer(n) => {
                        let value = -(*n);
                        self.advance();
                        if value < i16::MIN as i32 || value > i16::MAX as i32 {
                            Err(self.error(format!("Integer {} out of range for i16", value)))
                        } else {
                            Ok(value as i16)
                        }
                    }
                    _ => Err(self.error(format!(
                        "Expected integer after minus sign, found {}",
                        self.token_description(&self.current().kind)
                    ))),
                }
            }
            _ => Err(self.error(format!(
                "Expected integer, found {}",
                self.token_description(&self.current().kind)
            ))),
        }
    }

    fn parse_string(&mut self) -> Result<String, ParseError> {
        match &self.current().kind {
            TokenKind::String(s) => {
                let value = s.clone();
                self.advance();
                Ok(value)
            }
            _ => Err(self.error(format!(
                "Expected string, found {}",
                self.token_description(&self.current().kind)
            ))),
        }
    }

    fn error(&self, message: String) -> ParseError {
        ParseError::UnexpectedToken {
            expected: message,
            found: self.token_description(&self.current().kind),
            pos: self.current().pos,
        }
    }

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
            TokenKind::Room => "ROOM".to_string(),
            TokenKind::EndRoom => "ENDROOM".to_string(),
            TokenKind::Door => "DOOR".to_string(),
            TokenKind::EndDoor => "ENDDOOR".to_string(),
            TokenKind::Spot => "SPOT".to_string(),
            TokenKind::EndSpot => "ENDSPOT".to_string(),
            TokenKind::Script => "SCRIPT".to_string(),
            TokenKind::EndScript => "ENDSCRIPT".to_string(),
            TokenKind::Id => "ID".to_string(),
            TokenKind::Name => "NAME".to_string(),
            TokenKind::Pict => "PICT".to_string(),
            TokenKind::Artist => "ARTIST".to_string(),
            TokenKind::Dest => "DEST".to_string(),
            TokenKind::Outline => "OUTLINE".to_string(),
            TokenKind::Picts => "PICTS".to_string(),
            TokenKind::EndPicts => "ENDPICTS".to_string(),
            TokenKind::Picture => "PICTURE".to_string(),
            TokenKind::EndPicture => "ENDPICTURE".to_string(),
            TokenKind::TransColor => "TRANSCOLOR".to_string(),
            TokenKind::Private => "PRIVATE".to_string(),
            TokenKind::NoPainting => "NOPAINTING".to_string(),
            TokenKind::NoCyborgs => "NOCYBORGS".to_string(),
            TokenKind::Hidden => "HIDDEN".to_string(),
            TokenKind::NoGuests => "NOGUESTS".to_string(),
            TokenKind::Comma => ",".to_string(),
            TokenKind::Eof => "end of file".to_string(),
            _ => format!("{:?}", kind),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_room() {
        let source = r#"
ROOM
  ID 100
  NAME "Test Room"
  PICT "background.gif"
ENDROOM
"#;

        let mut parser = RoomScriptParser::new(source).unwrap();
        let rooms = parser.parse().unwrap();

        assert_eq!(rooms.len(), 1);
        assert_eq!(rooms[0].id, 100);
        assert_eq!(rooms[0].name, Some("Test Room".to_string()));
        assert_eq!(rooms[0].pict, Some("background.gif".to_string()));
    }

    #[test]
    fn test_parse_room_with_flags() {
        let source = r#"
ROOM
  ID 200
  NAME "Private Room"
  PRIVATE
  NOPAINTING
  HIDDEN
ENDROOM
"#;

        let mut parser = RoomScriptParser::new(source).unwrap();
        let rooms = parser.parse().unwrap();

        assert_eq!(rooms.len(), 1);
        assert!(rooms[0].flags.private);
        assert!(rooms[0].flags.no_painting);
        assert!(rooms[0].flags.hidden);
        assert!(!rooms[0].flags.no_cyborgs);
    }

    #[test]
    fn test_parse_door() {
        let source = r#"
ROOM
  ID 100
  DOOR
    ID 1
    DEST 200
    OUTLINE 10,10 50,10 50,200 10,200
  ENDDOOR
ENDROOM
"#;

        let mut parser = RoomScriptParser::new(source).unwrap();
        let rooms = parser.parse().unwrap();

        assert_eq!(rooms.len(), 1);
        assert_eq!(rooms[0].doors.len(), 1);
        assert_eq!(rooms[0].doors[0].id, 1);
        assert_eq!(rooms[0].doors[0].dest, 200);
        assert_eq!(rooms[0].doors[0].outline.len(), 4);
        assert_eq!(rooms[0].doors[0].outline[0], Point { h: 10, v: 10 });
    }

    #[test]
    fn test_parse_spot_with_outline() {
        let source = r#"
ROOM
  ID 100
  SPOT
    ID 2
    NAME "Button"
    OUTLINE 100,100 200,100 200,200 100,200
  ENDSPOT
ENDROOM
"#;

        let mut parser = RoomScriptParser::new(source).unwrap();
        let rooms = parser.parse().unwrap();

        assert_eq!(rooms.len(), 1);
        assert_eq!(rooms[0].spots.len(), 1);
        assert_eq!(rooms[0].spots[0].id, 2);
        assert_eq!(rooms[0].spots[0].name, Some("Button".to_string()));
        assert_eq!(rooms[0].spots[0].outline.len(), 4);
    }

    #[test]
    fn test_parse_picts() {
        let source = r#"
ROOM
  ID 100
  SPOT
    ID 2
    PICTS
      100,0,0
      101,10,-5
    ENDPICTS
  ENDSPOT
ENDROOM
"#;

        let mut parser = RoomScriptParser::new(source).unwrap();
        let rooms = parser.parse().unwrap();

        assert_eq!(rooms.len(), 1);
        assert_eq!(rooms[0].spots.len(), 1);
        assert_eq!(rooms[0].spots[0].picts.len(), 2);
        assert_eq!(rooms[0].spots[0].picts[0].pic_id, 100);
        assert_eq!(rooms[0].spots[0].picts[0].x_offset, 0);
        assert_eq!(rooms[0].spots[0].picts[0].y_offset, 0);
        assert_eq!(rooms[0].spots[0].picts[1].pic_id, 101);
        assert_eq!(rooms[0].spots[0].picts[1].x_offset, 10);
        assert_eq!(rooms[0].spots[0].picts[1].y_offset, -5);
    }

    #[test]
    fn test_parse_multiple_rooms() {
        let source = r#"
ROOM
  ID 100
  NAME "Room 1"
ENDROOM

ROOM
  ID 200
  NAME "Room 2"
ENDROOM
"#;

        let mut parser = RoomScriptParser::new(source).unwrap();
        let rooms = parser.parse().unwrap();

        assert_eq!(rooms.len(), 2);
        assert_eq!(rooms[0].id, 100);
        assert_eq!(rooms[1].id, 200);
    }

    #[test]
    fn test_parse_picture_decl() {
        let source = r#"
ROOM
  ID 100
  PICTURE
    ID 1
    NAME "overlay.gif"
    TRANSCOLOR 255
  ENDPICTURE
ENDROOM
"#;

        let mut parser = RoomScriptParser::new(source).unwrap();
        let rooms = parser.parse().unwrap();

        assert_eq!(rooms.len(), 1);
        assert_eq!(rooms[0].pictures.len(), 1);
        assert_eq!(rooms[0].pictures[0].id, 1);
        assert_eq!(rooms[0].pictures[0].name, "overlay.gif");
        assert_eq!(rooms[0].pictures[0].trans_color, Some(255));
    }

    #[test]
    fn test_parse_spot_with_script() {
        let source = r#"
ROOM
  ID 100
  SPOT
    ID 2
    SCRIPT
      ON SELECT {
        "You clicked!" SAY
      }
    ENDSCRIPT
  ENDSPOT
ENDROOM
"#;

        let mut parser = RoomScriptParser::new(source).unwrap();
        let rooms = parser.parse().unwrap();

        assert_eq!(rooms.len(), 1);
        assert_eq!(rooms[0].spots.len(), 1);
        assert!(rooms[0].spots[0].script.is_some());
        let script = rooms[0].spots[0].script.as_ref().unwrap();
        assert_eq!(script.handlers.len(), 1);
    }

    #[test]
    fn test_parse_negative_coordinates() {
        let source = r#"
ROOM
  ID 100
  SPOT
    ID 2
    OUTLINE -10,20 30,-40
  ENDSPOT
ENDROOM
"#;

        let mut parser = RoomScriptParser::new(source).unwrap();
        let rooms = parser.parse().unwrap();

        assert_eq!(rooms.len(), 1);
        assert_eq!(rooms[0].spots[0].outline.len(), 2);
        assert_eq!(rooms[0].spots[0].outline[0], Point { h: -10, v: 20 });
        assert_eq!(rooms[0].spots[0].outline[1], Point { h: 30, v: -40 });
    }
}
