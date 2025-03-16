#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Number(String),
    Keyword(String),
    Identifier(String),
    String(String),

    Semicolon,
    Colon,
    Arrow,

    LeftAngleBracket,
    RightAngleBracket,

    Equal,
    EqualEqual,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Plus,
    PlusEqual,
    Minus,
    MinusEqual,
    Star,
    Slash,
    Bang,

    Modulo,
    And,
    Or,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,

    Error(String),
}
