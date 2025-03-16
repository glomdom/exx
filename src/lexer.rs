use crate::position::Position;
use crate::span::Span;
use crate::token::{DiagnosticError, ErrorKind, Token};
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

    fn match_char(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.advance();

            true
        } else {
            false
        }
    }

    fn error_token(&self, start_pos: Position, kind: ErrorKind, message: &str) -> Token {
        Token {
            token_type: TokenType::Error(message.to_string()),
            span: Span::new(start_pos, self.current),
            errors: vec![DiagnosticError {
                kind,
                span: Span::new(start_pos, self.current),
            }],
        }
    }

    fn single_char_token(&mut self, start_pos: Position, token_type: TokenType) -> Token {
        let _ = self.advance();

        Token {
            token_type,
            span: Span::new(start_pos, self.current),
            errors: vec![],
        }
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

    fn read_sequence<F>(&mut self, mut predicate: F) -> String
    where
        F: FnMut(char) -> bool,
    {
        let mut sequence = String::new();

        while let Some(c) = self.peek() {
            if predicate(c) {
                let (ch, _) = self.advance().unwrap();
                sequence.push(ch);
            } else {
                break;
            }
        }

        sequence
    }

    fn read_number(&mut self, first_char: char, start_pos: Position) -> Token {
        let mut number = String::from(first_char);
        number.push_str(&self.read_sequence(|c| c.is_ascii_digit()));

        if self.match_char('.') {
            number.push('.');

            let fractional_part = self.read_sequence(|c| c.is_ascii_digit());
            if fractional_part.is_empty() {
                return self.error_token(
                    start_pos,
                    ErrorKind::InvalidDecimal,
                    "Invalid decimal number",
                );
            }

            number.push_str(&fractional_part);
        }

        Token {
            token_type: TokenType::Number(number),
            span: Span::new(start_pos, self.current),
            errors: vec![],
        }
    }

    fn read_identifier(&mut self, start_pos: Position) -> Token {
        let identifier = self.read_sequence(|c| c.is_ascii_alphanumeric() || c == '_');

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
        let mut operator = String::from(first_char);

        let valid_operators = [
            "==", "!=", "<=", ">=", "+=", "-=", "->", "&&", "||", "!", "<", ">", "+", "-", "*",
            "/", "%", "&", "|", "^", "=",
        ];

        while let Some(next_char) = self.peek() {
            if next_char.is_ascii_punctuation() {
                operator.push(next_char);
                self.advance();
            } else {
                break;
            }
        }

        if valid_operators.contains(&operator.as_str()) {
            let token_type = match operator.as_str() {
                "==" => TokenType::EqualEqual,
                "!=" => TokenType::NotEqual,
                "<=" => TokenType::LessEqual,
                ">=" => TokenType::GreaterEqual,
                "+=" => TokenType::PlusEqual,
                "-=" => TokenType::MinusEqual,
                "->" => TokenType::Arrow,
                "&&" => TokenType::And,
                "||" => TokenType::Or,
                "!" => TokenType::Bang,
                "<" => TokenType::Less,
                ">" => TokenType::Greater,
                "+" => TokenType::Plus,
                "-" => TokenType::Minus,
                "*" => TokenType::Star,
                "/" => TokenType::Slash,
                "%" => TokenType::Modulo,
                "&" => TokenType::BitwiseAnd,
                "|" => TokenType::BitwiseOr,
                "^" => TokenType::BitwiseXor,
                "=" => TokenType::Equal,

                _ => unreachable!(),
            };

            Token {
                token_type,
                span: Span::new(start_pos, self.current),
                errors: vec![],
            }
        } else {
            self.error_token(
                start_pos,
                ErrorKind::InvalidOperator(operator.clone()),
                &format!("Invalid operator: `{}`", operator),
            )
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
                                    kind: ErrorKind::InvalidEscape(escaped),
                                    span: Span::new(escape_start, self.current),
                                });

                                string.push(escaped);
                            }
                        }
                    } else {
                        errors.push(DiagnosticError {
                            kind: ErrorKind::UnterminatedEscapeSequence,
                            span: Span::new(escape_start, self.current),
                        });

                        break;
                    }
                }

                _ => string.push(c),
            }
        }

        errors.push(DiagnosticError {
            kind: ErrorKind::UnterminatedString,
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

            'a'..='z' | 'A'..='Z' | '_' => self.read_identifier(start_pos),

            '+' | '-' | '*' | '/' | '=' | '<' | '>' | '!' | '&' | '^' | '%' | '|' => {
                self.read_operator(start_pos)
            }

            '"' => {
                let _ = self.advance()?;

                self.read_string(start_pos)
            }

            ';' => self.single_char_token(start_pos, TokenType::Semicolon),
            ':' => self.single_char_token(start_pos, TokenType::Colon),
            '(' => self.single_char_token(start_pos, TokenType::LeftParen),
            ')' => self.single_char_token(start_pos, TokenType::RightParen),

            c if c.is_whitespace() => return self.next(),

            _ => {
                let _ = self.advance()?;

                Token {
                    token_type: TokenType::Error(format!("Unexpected character: '{}'", first_char)),
                    span: Span::new(start_pos, self.current),
                    errors: vec![DiagnosticError {
                        kind: ErrorKind::UnexpectedCharacter(first_char),
                        span: Span::new(start_pos, self.current),
                    }],
                }
            }
        };

        Some(token)
    }
}
