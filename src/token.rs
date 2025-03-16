use crate::span::Span;
use crate::tokentype::TokenType;

#[derive(Debug, Clone)]
pub enum ErrorKind {
    UnexpectedCharacter(char),
    InvalidDecimal,
    InvalidOperator(String),
    InvalidEscape(char),
    UnterminatedString,
    UnterminatedBlockComment,
}

impl ErrorKind {
    pub fn code(&self) -> String {
        match self {
            Self::UnexpectedCharacter(_) => "E000".into(),
            Self::InvalidDecimal => "E001".into(),
            Self::InvalidOperator(_) => "E002".into(),
            Self::InvalidEscape(_) => "E003".into(),
            Self::UnterminatedString => "E004".into(),
            Self::UnterminatedBlockComment => "E005".into(),
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::UnexpectedCharacter(c) => format!("Unexpected character: '{}'", c),
            Self::InvalidDecimal => "Invalid decimal number".into(),
            Self::InvalidOperator(op) => format!("Invalid operator: {}", op),
            Self::InvalidEscape(c) => format!("Invalid escape sequence: \\{}", c),
            Self::UnterminatedString => "Unterminated string literal".into(),
            Self::UnterminatedBlockComment => "Unterminated block comment".into(),
        }
    }

    pub fn label(&self) -> String {
        match self {
            Self::UnexpectedCharacter(_) => "Unexpected character".into(),
            Self::InvalidDecimal => "Expected digits after decimal point".into(),
            Self::InvalidOperator(_) => "Invalid operator".into(),
            Self::InvalidEscape(_) => "Invalid escape sequence".into(),
            Self::UnterminatedString => "Unterminated string".into(),
            Self::UnterminatedBlockComment => "Unterminated block comment".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DiagnosticError {
    pub kind: ErrorKind,
    pub span: Span,
}

impl DiagnosticError {
    pub fn new(kind: ErrorKind, span: Span) -> Self {
        Self { kind, span }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub span: Span,
    pub errors: Vec<DiagnosticError>,
}
