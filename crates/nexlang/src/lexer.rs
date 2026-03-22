//! NEXLANG Lexer - Tokenizes source code.

use crate::errors::NexlangError;

/// Token kinds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    // Literals
    IntLiteral(u128),
    StringLiteral(String),
    BoolLiteral(bool),

    // Identifiers & Keywords
    Identifier(String),
    Contract, Fn, Let, Mut, If, Else, While, For, In, Return,
    Pub, Priv, Internal,
    True, False,
    Event, Emit, Require,
    SelfKw, MsgSender, MsgValue, BlockHeight,
    U8, U16, U32, U64, U128, U256, I64, I128,
    Bool, StringType, Address, Bytes, Map, Array,

    // Annotations
    At, // @

    // Operators
    Plus, Minus, Star, Slash, Percent,
    Eq, Ne, Lt, Gt, Le, Ge,
    And, Or, Not,
    Assign, Arrow,
    Dot, Comma, Colon, Semicolon,
    LParen, RParen, LBrace, RBrace, LBracket, RBracket,

    // Special
    Newline,
    Eof,
}

/// A token with position information.
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub col: usize,
}

/// NEXLANG Lexer.
pub struct Lexer {
    source: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
}

impl Lexer {
    /// Create a new lexer for the given source code.
    pub fn new(source: &str) -> Self {
        Lexer {
            source: source.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    /// Tokenize the entire source.
    pub fn tokenize(&mut self) -> Result<Vec<Token>, NexlangError> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token()?;
            let is_eof = tok.kind == TokenKind::Eof;
            tokens.push(tok);
            if is_eof {
                break;
            }
        }
        Ok(tokens)
    }

    /// Get the next token.
    pub fn next_token(&mut self) -> Result<Token, NexlangError> {
        self.skip_whitespace();
        self.skip_comments();
        self.skip_whitespace();

        if self.pos >= self.source.len() {
            return Ok(Token { kind: TokenKind::Eof, line: self.line, col: self.col });
        }

        let ch = self.source[self.pos];
        let line = self.line;
        let col = self.col;

        // Newline
        if ch == '\n' {
            self.advance();
            return Ok(Token { kind: TokenKind::Newline, line, col });
        }

        // String literal
        if ch == '"' {
            return self.read_string();
        }

        // Number literal
        if ch.is_ascii_digit() {
            return self.read_number();
        }

        // Identifier or keyword
        if ch.is_alphabetic() || ch == '_' {
            return self.read_identifier();
        }

        // Operators and punctuation
        let kind = match ch {
            '@' => { self.advance(); TokenKind::At }
            '+' => { self.advance(); TokenKind::Plus }
            '-' => {
                self.advance();
                if self.peek() == Some('>') { self.advance(); TokenKind::Arrow }
                else { TokenKind::Minus }
            }
            '*' => { self.advance(); TokenKind::Star }
            '/' => { self.advance(); TokenKind::Slash }
            '%' => { self.advance(); TokenKind::Percent }
            '=' => {
                self.advance();
                if self.peek() == Some('=') { self.advance(); TokenKind::Eq }
                else { TokenKind::Assign }
            }
            '!' => {
                self.advance();
                if self.peek() == Some('=') { self.advance(); TokenKind::Ne }
                else { TokenKind::Not }
            }
            '<' => {
                self.advance();
                if self.peek() == Some('=') { self.advance(); TokenKind::Le }
                else { TokenKind::Lt }
            }
            '>' => {
                self.advance();
                if self.peek() == Some('=') { self.advance(); TokenKind::Ge }
                else { TokenKind::Gt }
            }
            '&' => {
                self.advance();
                if self.peek() == Some('&') { self.advance(); TokenKind::And }
                else { return Err(self.error("Expected '&&'")); }
            }
            '|' => {
                self.advance();
                if self.peek() == Some('|') { self.advance(); TokenKind::Or }
                else { return Err(self.error("Expected '||'")); }
            }
            '.' => { self.advance(); TokenKind::Dot }
            ',' => { self.advance(); TokenKind::Comma }
            ':' => { self.advance(); TokenKind::Colon }
            ';' => { self.advance(); TokenKind::Semicolon }
            '(' => { self.advance(); TokenKind::LParen }
            ')' => { self.advance(); TokenKind::RParen }
            '{' => { self.advance(); TokenKind::LBrace }
            '}' => { self.advance(); TokenKind::RBrace }
            '[' => { self.advance(); TokenKind::LBracket }
            ']' => { self.advance(); TokenKind::RBracket }
            _ => return Err(self.error(&format!("Unexpected character: '{}'", ch))),
        };

        Ok(Token { kind, line, col })
    }

    fn advance(&mut self) -> Option<char> {
        if self.pos < self.source.len() {
            let ch = self.source[self.pos];
            self.pos += 1;
            if ch == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
            Some(ch)
        } else {
            None
        }
    }

    fn peek(&self) -> Option<char> {
        self.source.get(self.pos).copied()
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.source.len() {
            let ch = self.source[self.pos];
            if ch == ' ' || ch == '\t' || ch == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_comments(&mut self) {
        if self.pos + 1 < self.source.len()
            && self.source[self.pos] == '/'
            && self.source[self.pos + 1] == '/'
        {
            while self.pos < self.source.len() && self.source[self.pos] != '\n' {
                self.advance();
            }
        }
    }

    fn read_string(&mut self) -> Result<Token, NexlangError> {
        let line = self.line;
        let col = self.col;
        self.advance(); // skip opening quote
        let mut s = String::new();
        while self.pos < self.source.len() && self.source[self.pos] != '"' {
            if self.source[self.pos] == '\\' {
                self.advance();
                match self.peek() {
                    Some('n') => { s.push('\n'); self.advance(); }
                    Some('t') => { s.push('\t'); self.advance(); }
                    Some('"') => { s.push('"'); self.advance(); }
                    Some('\\') => { s.push('\\'); self.advance(); }
                    _ => return Err(self.error("Invalid escape sequence")),
                }
            } else {
                s.push(self.source[self.pos]);
                self.advance();
            }
        }
        if self.pos >= self.source.len() {
            return Err(NexlangError::LexerError { line, col, message: "Unterminated string".into() });
        }
        self.advance(); // skip closing quote
        Ok(Token { kind: TokenKind::StringLiteral(s), line, col })
    }

    fn read_number(&mut self) -> Result<Token, NexlangError> {
        let line = self.line;
        let col = self.col;
        let mut num_str = String::new();
        while self.pos < self.source.len() && self.source[self.pos].is_ascii_digit() {
            num_str.push(self.source[self.pos]);
            self.advance();
        }
        // Skip underscores in number literals
        num_str.retain(|c| c != '_');
        let value: u128 = num_str.parse().map_err(|_| {
            NexlangError::LexerError { line, col, message: format!("Invalid number: {}", num_str) }
        })?;
        Ok(Token { kind: TokenKind::IntLiteral(value), line, col })
    }

    fn read_identifier(&mut self) -> Result<Token, NexlangError> {
        let line = self.line;
        let col = self.col;
        let mut ident = String::new();
        while self.pos < self.source.len()
            && (self.source[self.pos].is_alphanumeric() || self.source[self.pos] == '_')
        {
            ident.push(self.source[self.pos]);
            self.advance();
        }
        let kind = match ident.as_str() {
            "contract" => TokenKind::Contract,
            "fn" => TokenKind::Fn,
            "let" => TokenKind::Let,
            "mut" => TokenKind::Mut,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "for" => TokenKind::For,
            "in" => TokenKind::In,
            "return" => TokenKind::Return,
            "pub" => TokenKind::Pub,
            "priv" => TokenKind::Priv,
            "internal" => TokenKind::Internal,
            "true" => TokenKind::BoolLiteral(true),
            "false" => TokenKind::BoolLiteral(false),
            "event" => TokenKind::Event,
            "emit" => TokenKind::Emit,
            "require" => TokenKind::Require,
            "self" => TokenKind::SelfKw,
            "msg_sender" => TokenKind::MsgSender,
            "msg_value" => TokenKind::MsgValue,
            "block_height" => TokenKind::BlockHeight,
            "u8" => TokenKind::U8,
            "u16" => TokenKind::U16,
            "u32" => TokenKind::U32,
            "u64" => TokenKind::U64,
            "u128" => TokenKind::U128,
            "u256" => TokenKind::U256,
            "i64" => TokenKind::I64,
            "i128" => TokenKind::I128,
            "bool" => TokenKind::Bool,
            "String" => TokenKind::StringType,
            "Address" => TokenKind::Address,
            "Bytes" => TokenKind::Bytes,
            "Map" => TokenKind::Map,
            "Array" => TokenKind::Array,
            _ => TokenKind::Identifier(ident),
        };
        Ok(Token { kind, line, col })
    }

    fn error(&self, msg: &str) -> NexlangError {
        NexlangError::LexerError {
            line: self.line,
            col: self.col,
            message: msg.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let mut lexer = Lexer::new("contract Token { }");
        let tokens = lexer.tokenize().unwrap();
        assert!(matches!(tokens[0].kind, TokenKind::Contract));
        assert!(matches!(&tokens[1].kind, TokenKind::Identifier(n) if n == "Token"));
        assert!(matches!(tokens[2].kind, TokenKind::LBrace));
        assert!(matches!(tokens[3].kind, TokenKind::RBrace));
    }

    #[test]
    fn test_number_literal() {
        let mut lexer = Lexer::new("42");
        let tokens = lexer.tokenize().unwrap();
        assert!(matches!(tokens[0].kind, TokenKind::IntLiteral(42)));
    }

    #[test]
    fn test_string_literal() {
        let mut lexer = Lexer::new("\"hello\"");
        let tokens = lexer.tokenize().unwrap();
        assert!(matches!(&tokens[0].kind, TokenKind::StringLiteral(s) if s == "hello"));
    }

    #[test]
    fn test_operators() {
        let mut lexer = Lexer::new("a + b == c && d != e");
        let tokens = lexer.tokenize().unwrap();
        assert!(matches!(&tokens[0].kind, TokenKind::Identifier(n) if n == "a"));
        assert!(matches!(tokens[1].kind, TokenKind::Plus));
        assert!(matches!(tokens[3].kind, TokenKind::Eq));
        assert!(matches!(tokens[5].kind, TokenKind::And));
        assert!(matches!(tokens[7].kind, TokenKind::Ne));
    }

    #[test]
    fn test_annotation() {
        let mut lexer = Lexer::new("@nonreentrant");
        let tokens = lexer.tokenize().unwrap();
        assert!(matches!(tokens[0].kind, TokenKind::At));
        assert!(matches!(&tokens[1].kind, TokenKind::Identifier(n) if n == "nonreentrant"));
    }

    #[test]
    fn test_keywords() {
        let mut lexer = Lexer::new("fn let mut if else while return pub");
        let tokens = lexer.tokenize().unwrap();
        assert!(matches!(tokens[0].kind, TokenKind::Fn));
        assert!(matches!(tokens[1].kind, TokenKind::Let));
        assert!(matches!(tokens[2].kind, TokenKind::Mut));
        assert!(matches!(tokens[3].kind, TokenKind::If));
    }

    #[test]
    fn test_comment_skip() {
        let mut lexer = Lexer::new("// this is a comment\n42");
        let tokens = lexer.tokenize().unwrap();
        // Should skip comment, get newline then 42
        let non_newline: Vec<_> = tokens.iter()
            .filter(|t| !matches!(t.kind, TokenKind::Newline | TokenKind::Eof))
            .collect();
        assert_eq!(non_newline.len(), 1);
        assert!(matches!(non_newline[0].kind, TokenKind::IntLiteral(42)));
    }
}
