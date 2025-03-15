use crate::{span::Span, tokentype::TokenType};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub span: Span,
}
