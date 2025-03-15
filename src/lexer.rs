use crate::position::Position;
use crate::span::Span;
use crate::token::Token;
use crate::tokentype::TokenType;
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug)]
pub struct Lexer<'src> {
    input: Peekable<Chars<'src>>,
    current: Position,
    error_encountered: bool,
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
            error_encountered: false,
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

    fn read_number(&mut self, first_char: char, start_pos: Position) -> (TokenType, Span) {
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
                return (
                    TokenType::Error("Invalid decimal number".to_string()),
                    Span::new(start_pos, end_pos),
                );
            }
        }

        (TokenType::Number(number), Span::new(start_pos, end_pos))
    }

    fn read_identifier(&mut self, first_char: char, start_pos: Position) -> (TokenType, Span) {
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

        (token_type, Span::new(start_pos, end_pos))
    }

    fn read_operator(&mut self, first_char: char, start_pos: Position) -> (TokenType, Span) {
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
            _ => TokenType::Error(format!("Invalid operator: {}", operator)),
        };

        (token_type, Span::new(start_pos, end_pos))
    }

    fn read_string(&mut self, start_pos: Position) -> (TokenType, Span) {
        let mut string = String::new();
        let mut end_pos = start_pos;
        let mut error = None;
        let mut error_span = None;

        while let Some((c, pos)) = self.advance() {
            match c {
                '"' => {
                    end_pos = pos;
                    return (TokenType::String(string), Span::new(start_pos, end_pos));
                }
                '\\' => {
                    let escape_start = pos;

                    match self.advance() {
                        Some((escaped, esc_pos)) => {
                            end_pos = esc_pos;

                            match escaped {
                                'n' => string.push('\n'),
                                't' => string.push('\t'),
                                'r' => string.push('\r'),
                                '\\' => string.push('\\'),
                                '"' => string.push('"'),
                                _ => {
                                    error = Some(format!("Invalid escape sequence: \\{}", escaped));
                                    error_span = Some(Span::new(escape_start, esc_pos));
                                }
                            }
                        }

                        None => {
                            error = Some("Unterminated escape sequence".to_string());
                            error_span = Some(Span::new(escape_start, pos));

                            break;
                        }
                    }
                }
                _ => {
                    string.push(c);
                    end_pos = pos;
                }
            }

            if error.is_some() {
                break;
            }
        }

        if let Some(err) = error {
            (
                TokenType::Error(err),
                error_span.unwrap_or(Span::new(start_pos, end_pos)),
            )
        } else {
            (
                TokenType::Error("Unterminated string".to_string()),
                Span::new(start_pos, end_pos),
            )
        }
    }
}

impl Iterator for Lexer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.error_encountered {
            return None;
        }

        self.skip_whitespace();
        let (first_char, start_pos) = self.advance()?;

        let (token_type, span) = match first_char {
            '0'..='9' => self.read_number(first_char, start_pos),
            'a'..='z' | 'A'..='Z' | '_' => self.read_identifier(first_char, start_pos),
            '+' | '-' | '*' | '/' | '=' | '<' | '>' | '!' | '&' | '^' | '%' | '|' => {
                self.read_operator(first_char, start_pos)
            }
            '"' => self.read_string(start_pos),
            ';' => (TokenType::Semicolon, Span::new(start_pos, start_pos)),
            c if c.is_whitespace() => return self.next(),
            _ => (
                TokenType::Error(format!("Unexpected character: '{}'", first_char)),
                Span::new(start_pos, start_pos),
            ),
        };

        if let TokenType::Error(_) = token_type {
            self.error_encountered = true;
        }

        Some(Token { token_type, span })
    }
}
