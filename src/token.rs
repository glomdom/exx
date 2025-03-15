use crate::span::Span;
use crate::tokentype::TokenType;

#[derive(Debug, Clone)]
pub struct DiagnosticError {
    pub error_code: Option<String>,
    pub message: String,
    pub label: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub span: Span,
    pub errors: Vec<DiagnosticError>,
}
