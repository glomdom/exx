use crate::position::Position;
use crate::span::Span;
use crate::token::{DiagnosticError, Token};
use crate::tokentype::TokenType;
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug)]
pub struct Lexer<'src> {
    input: Peekable<Chars<'src>>,
    current: Position,
}

impl<'src> Lexer<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
            input: source.chars().peekable(),
            current: Position {
                line: 1,
                column: 1,
                absolute: 0,
            },
        }
    }

    fn advance(&mut self) -> Option<(char, Position)> {
        let start_pos = self.current;
        let c = self.input.next()?;

        let byte_len = c.len_utf8();
        self.current.absolute += byte_len;

        if c == '\n' {
            self.current.line += 1;
            self.current.column = 1;
        } else {
            self.current.column += 1;
        }

        Some((c, start_pos))
    }

    fn peek(&mut self) -> Option<char> {
        self.input.peek().copied()
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_number(&mut self, first_char: char, start_pos: Position) -> Token {
        let mut number = String::from(first_char);

        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                let (ch, _) = self.advance().unwrap();
                number.push(ch);
            } else {
                break;
            }
        }

        if self.peek() == Some('.') {
            let _ = self.advance().unwrap(); // consume '.'
            number.push('.');

            let mut has_digit = false;

            while let Some(c) = self.peek() {
                if c.is_ascii_digit() {
                    let (ch, _) = self.advance().unwrap();
                    number.push(ch);
                    has_digit = true;
                } else {
                    break;
                }
            }

            if !has_digit {
                let span = Span::new(start_pos, self.current);

                return Token {
                    token_type: TokenType::Error("Invalid decimal number".to_string()),
                    span: span.clone(),
                    errors: vec![DiagnosticError {
                        error_code: Some("E001".to_string()),
                        message: "Invalid decimal number".to_string(),
                        label: Some("Expected digits after decimal point".to_string()),
                        span,
                    }],
                };
            }
        }

        Token {
            token_type: TokenType::Number(number),
            span: Span::new(start_pos, self.current),
            errors: vec![],
        }
    }

    fn read_identifier(&mut self, first_char: char, start_pos: Position) -> Token {
        let mut identifier = String::from(first_char);
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == '_' {
                let (ch, _) = self.advance().unwrap();
                identifier.push(ch);
            } else {
                break;
            }
        }

        let token_type = match identifier.as_str() {
            "let" | "var" | "type" | "if" | "else" | "return" => TokenType::Keyword(identifier),

            _ => TokenType::Identifier(identifier),
        };

        Token {
            token_type,
            span: Span::new(start_pos, self.current),
            errors: vec![],
        }
    }

    fn read_operator(&mut self, start_pos: Position) -> Token {
        let (first_char, _) = self.advance().unwrap();
        let token_type = match first_char {
            '=' => match self.peek() {
                Some('=') => {
                    self.advance();
                    TokenType::EqualEqual
                }
                _ => TokenType::Equal,
            },

            '!' => match self.peek() {
                Some('=') => {
                    self.advance();
                    TokenType::NotEqual
                }
                _ => TokenType::Bang,
            },

            '<' => match self.peek() {
                Some('=') => {
                    self.advance();
                    TokenType::LessEqual
                }
                _ => TokenType::Less,
            },

            '>' => match self.peek() {
                Some('=') => {
                    self.advance();
                    TokenType::GreaterEqual
                }
                _ => TokenType::Greater,
            },

            '+' => match self.peek() {
                Some('=') => {
                    self.advance();
                    TokenType::PlusEqual
                }
                _ => TokenType::Plus,
            },

            '-' => match self.peek() {
                Some('>') => {
                    self.advance();
                    TokenType::Arrow
                }
                Some('=') => {
                    self.advance();
                    TokenType::MinusEqual
                }
                _ => TokenType::Minus,
            },

            '*' => TokenType::Star,
            '/' => TokenType::Slash,
            '%' => TokenType::Modulo,
            '&' => match self.peek() {
                Some('&') => {
                    self.advance();
                    TokenType::And
                }
                _ => TokenType::BitwiseAnd,
            },

            '|' => match self.peek() {
                Some('|') => {
                    self.advance();
                    TokenType::Or
                }
                _ => TokenType::BitwiseOr,
            },

            '^' => TokenType::BitwiseXor,

            _ => TokenType::Error(format!("Invalid operator: {}", first_char)),
        };

        Token {
            token_type,
            span: Span::new(start_pos, self.current),
            errors: vec![],
        }
    }

    fn read_string(&mut self, start_pos: Position) -> Token {
        let mut string = String::new();
        let mut errors = Vec::new();

        while let Some((c, _)) = self.advance() {
            match c {
                '"' => {
                    return Token {
                        token_type: TokenType::String(string),
                        span: Span::new(start_pos, self.current),
                        errors,
                    };
                }

                '\\' => {
                    let escape_start = self.current;

                    if let Some((escaped, _)) = self.advance() {
                        match escaped {
                            'n' => string.push('\n'),
                            't' => string.push('\t'),
                            'r' => string.push('\r'),
                            '\\' => string.push('\\'),
                            '"' => string.push('"'),

                            _ => {
                                errors.push(DiagnosticError {
                                    error_code: Some("E003".to_string()),
                                    message: format!("Invalid escape sequence: \\{}", escaped),
                                    label: Some("Invalid escape sequence".to_string()),
                                    span: Span::new(escape_start, self.current),
                                });

                                string.push(escaped);
                            }
                        }
                    } else {
                        errors.push(DiagnosticError {
                            error_code: Some("E003".to_string()),
                            message: "Unterminated escape sequence".to_string(),
                            label: Some("Unterminated escape sequence".to_string()),
                            span: Span::new(escape_start, self.current),
                        });

                        break;
                    }
                }
                _ => {
                    string.push(c);
                }
            }
        }

        errors.push(DiagnosticError {
            error_code: Some("E004".to_string()),
            message: "Unterminated string literal".to_string(),
            label: Some("Unterminated string".to_string()),
            span: Span::new(start_pos, self.current),
        });

        Token {
            token_type: TokenType::Error("String literal error".to_string()),
            span: Span::new(start_pos, self.current),
            errors,
        }
    }
}

impl Iterator for Lexer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();

        let start_pos = self.current;
        let first_char = self.peek()?;

        let token = match first_char {
            '0'..='9' => {
                let (ch, _) = self.advance()?;

                self.read_number(ch, start_pos)
            }

            'a'..='z' | 'A'..='Z' | '_' => {
                let (ch, _) = self.advance()?;

                self.read_identifier(ch, start_pos)
            }

            '+' | '-' | '*' | '/' | '=' | '<' | '>' | '!' | '&' | '^' | '%' | '|' => {
                self.read_operator(start_pos)
            }

            '"' => {
                let _ = self.advance()?;

                self.read_string(start_pos)
            }

            ';' => {
                let _ = self.advance()?;

                Token {
                    token_type: TokenType::Semicolon,
                    span: Span::new(start_pos, self.current),
                    errors: vec![],
                }
            }

            c if c.is_whitespace() => return self.next(),
            _ => {
                let _ = self.advance()?;

                Token {
                    token_type: TokenType::Error(format!("Unexpected character: '{}'", first_char)),
                    span: Span::new(start_pos, self.current),
                    errors: vec![DiagnosticError {
                        error_code: Some("E000".to_string()),
                        message: format!("Unexpected character: '{}'", first_char),
                        label: Some("Unexpected character".to_string()),
                        span: Span::new(start_pos, self.current),
                    }],
                }
            }
        };

        Some(token)
    }
}
