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
        let mut end_pos = start_pos;

        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                let (ch, pos) = self.advance().unwrap();
                number.push(ch);
                end_pos = pos;
            } else {
                break;
            }
        }

        if self.peek() == Some('.') {
            let (_, dot_pos) = self.advance().unwrap();
            number.push('.');
            end_pos = dot_pos;

            let mut has_digit = false;

            while let Some(c) = self.peek() {
                if c.is_ascii_digit() {
                    let (ch, pos) = self.advance().unwrap();
                    number.push(ch);
                    end_pos = pos;
                    has_digit = true;
                } else {
                    break;
                }
            }

            if !has_digit {
                return Token {
                    token_type: TokenType::Error("Invalid decimal number".to_string()),
                    span: Span::new(start_pos, end_pos),
                    errors: vec![DiagnosticError {
                        error_code: Some("E001".to_string()),
                        message: "Invalid decimal number".to_string(),
                        label: Some("Expected digits after decimal point".to_string()),
                        span: Span::new(start_pos, end_pos),
                    }],
                };
            }
        }

        Token {
            token_type: TokenType::Number(number),
            span: Span::new(start_pos, end_pos),
            errors: vec![],
        }
    }

    fn read_identifier(&mut self, first_char: char, start_pos: Position) -> Token {
        let mut identifier = String::from(first_char);
        let mut end_pos = start_pos;

        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == '_' {
                let (ch, pos) = self.advance().unwrap();
                identifier.push(ch);
                end_pos = pos;
            } else {
                break;
            }
        }

        let token_type = match identifier.as_str() {
            "let" | "if" | "else" | "fn" | "return" => TokenType::Keyword(identifier),

            _ => TokenType::Identifier(identifier),
        };

        Token {
            token_type,
            span: Span::new(start_pos, end_pos),
            errors: vec![],
        }
    }

    fn read_operator(&mut self, first_char: char, start_pos: Position) -> Token {
        let mut operator = String::from(first_char);
        let mut end_pos = start_pos;

        while let Some(c) = self.peek() {
            if "+-*/=<>!&|^%".contains(c) {
                let (ch, pos) = self.advance().unwrap();
                operator.push(ch);
                end_pos = pos;
            } else {
                break;
            }
        }

        let token_type = match operator.as_str() {
            "=" => TokenType::Equal,
            "==" => TokenType::EqualEqual,
            "!=" => TokenType::NotEqual,
            "!" => TokenType::Bang,
            "<" => TokenType::Less,
            "<=" => TokenType::LessEqual,
            ">" => TokenType::Greater,
            ">=" => TokenType::GreaterEqual,
            "+" => TokenType::Plus,
            "+=" => TokenType::PlusEqual,
            "-" => TokenType::Minus,
            "-=" => TokenType::MinusEqual,
            "*" => TokenType::Star,
            "/" => TokenType::Slash,
            "%" => TokenType::Modulo,
            "&&" => TokenType::And,
            "||" => TokenType::Or,
            "&" => TokenType::BitwiseAnd,
            "|" => TokenType::BitwiseOr,
            "^" => TokenType::BitwiseXor,

            _ => {
                return Token {
                    token_type: TokenType::Error(format!("Invalid operator: {}", operator)),
                    span: Span::new(start_pos, end_pos),
                    errors: vec![DiagnosticError {
                        error_code: Some("E002".to_string()),
                        message: format!("Invalid operator: {}", operator),
                        label: Some("Invalid operator".to_string()),
                        span: Span::new(start_pos, end_pos),
                    }],
                };
            }
        };

        Token {
            token_type,
            span: Span::new(start_pos, end_pos),
            errors: vec![],
        }
    }

    fn read_string(&mut self, start_pos: Position) -> Token {
        let mut string = String::new();
        let mut end_pos = start_pos;
        let mut errors = Vec::new();

        while let Some((c, pos)) = self.advance() {
            match c {
                '"' => {
                    end_pos = pos;
                    return Token {
                        token_type: TokenType::String(string),
                        span: Span::new(start_pos, end_pos),
                        errors,
                    };
                }

                '\\' => {
                    let escape_start = pos;

                    if let Some((escaped, esc_pos)) = self.advance() {
                        end_pos = esc_pos;

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
                                    span: Span::new(escape_start, esc_pos),
                                });

                                string.push(escaped);
                            }
                        }
                    } else {
                        errors.push(DiagnosticError {
                            error_code: Some("E003".to_string()),
                            message: "Unterminated escape sequence".to_string(),
                            label: Some("Unterminated escape sequence".to_string()),
                            span: Span::new(escape_start, pos),
                        });

                        break;
                    }
                }

                _ => {
                    string.push(c);
                    end_pos = pos;
                }
            }
        }

        errors.push(DiagnosticError {
            error_code: Some("E004".to_string()),
            message: "Unterminated string literal".to_string(),
            label: Some("Unterminated string".to_string()),
            span: Span::new(start_pos, end_pos),
        });

        Token {
            token_type: TokenType::Error("String literal error".to_string()),
            span: Span::new(start_pos, end_pos),
            errors,
        }
    }
}

impl Iterator for Lexer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();

        let (first_char, start_pos) = self.advance()?;
        let token = match first_char {
            '0'..='9' => self.read_number(first_char, start_pos),
            'a'..='z' | 'A'..='Z' | '_' => self.read_identifier(first_char, start_pos),
            '+' | '-' | '*' | '/' | '=' | '<' | '>' | '!' | '&' | '^' | '%' | '|' => {
                self.read_operator(first_char, start_pos)
            }

            '"' => self.read_string(start_pos),
            ';' => Token {
                token_type: TokenType::Semicolon,
                span: Span::new(start_pos, start_pos),
                errors: vec![],
            },

            c if c.is_whitespace() => return self.next(),

            _ => Token {
                token_type: TokenType::Error(format!("Unexpected character: '{}'", first_char)),
                span: Span::new(start_pos, start_pos),
                errors: vec![DiagnosticError {
                    error_code: Some("E000".to_string()),
                    message: format!("Unexpected character: '{}'", first_char),
                    label: Some("Unexpected character".to_string()),
                    span: Span::new(start_pos, start_pos),
                }],
            },
        };

        Some(token)
    }
}
