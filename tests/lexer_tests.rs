use exx::{
    lexer::Lexer,
    token::{ErrorKind, Token},
    tokentype::TokenType,
};
use pretty_assertions::assert_eq;

fn lex_all(source: &str) -> Vec<Token> {
    Lexer::new(source).collect()
}

#[test]
fn test_number_token() {
    let source = "123";
    let tokens = lex_all(source);

    assert_eq!(tokens.len(), 1);

    match &tokens[0].token_type {
        TokenType::Number(num) => {
            assert_eq!(num, "123");
            assert_eq!(tokens[0].span.start.absolute, 0);
            assert_eq!(tokens[0].span.end.absolute, 3);
        }

        _ => panic!("Expected number token"),
    }
}

#[test]
fn test_identifier_token() {
    let source = "letVar";
    let tokens = lex_all(source);

    assert_eq!(tokens.len(), 1);

    match &tokens[0].token_type {
        TokenType::Identifier(ident) => {
            assert_eq!(ident, "letVar");
            assert_eq!(tokens[0].span.start.absolute, 0);
            assert_eq!(tokens[0].span.end.absolute, 6);
        }

        _ => panic!("Expected identifier token"),
    }
}

#[test]
fn test_operator_token() {
    let source = "==";
    let tokens = lex_all(source);

    assert_eq!(tokens.len(), 1);

    match tokens[0].token_type {
        TokenType::EqualEqual => {
            assert_eq!(tokens[0].span.start.absolute, 0);
            assert_eq!(tokens[0].span.end.absolute, 2);
        }

        _ => panic!("Expected '==' operator token"),
    }
}

#[test]
fn test_string_token() {
    let source = "\"abc\"";
    let tokens = lex_all(source);

    assert_eq!(tokens.len(), 1);

    match &tokens[0].token_type {
        TokenType::String(s) => {
            assert_eq!(s, "abc");
            assert_eq!(tokens[0].span.start.absolute, 0);
            assert_eq!(tokens[0].span.end.absolute, 5);
        }

        _ => panic!("Expected string token"),
    }
}

#[test]
fn test_unterminated_string_error() {
    let source = "\"abc";
    let tokens = lex_all(source);

    assert_eq!(tokens.len(), 1);

    match &tokens[0].token_type {
        TokenType::Error(msg) => {
            assert_eq!(msg.contains("String literal error"), true);
            assert_eq!(tokens[0].span.start.absolute, 0);
            assert_eq!(tokens[0].span.end.absolute, source.len());
        }

        _ => panic!("Expected error token for unterminated string"),
    }
}

#[test]
fn test_invalid_operator() {
    let source = "!=!!!==!!";
    let tokens = lex_all(source);

    assert_eq!(tokens.len(), 1);

    match &tokens[0].token_type {
        TokenType::Error(msg) => {
            assert_eq!(msg.contains("Invalid operator"), true);
            assert_eq!(tokens[0].span.start.absolute, 0);
            assert_eq!(tokens[0].span.end.absolute, 9);
        }

        _ => panic!("Expected error token for invalid operator"),
    }
}

#[test]
fn test_valid_compound_operator() {
    let source = "&&";
    let tokens = lex_all(source);

    assert_eq!(tokens.len(), 1);

    match tokens[0].token_type {
        TokenType::And => {
            assert_eq!(tokens[0].span.start.absolute, 0);
            assert_eq!(tokens[0].span.end.absolute, 2);
        }

        _ => panic!("Expected '&&' operator token"),
    }
}

#[test]
fn test_position_tracking_newline() {
    let source = "123\n456";
    let tokens = lex_all(source);

    assert_eq!(tokens.len(), 2);

    assert_eq!(tokens[0].span.start.line, 1);
    assert_eq!(tokens[0].span.end.line, 1);
    assert_eq!(tokens[0].span.end.column, 4);

    assert_eq!(tokens[1].span.start.line, 2);
    assert_eq!(tokens[1].span.start.column, 1);
}

#[test]
fn test_position_tracking_columns() {
    let source = "a b c";
    let tokens = lex_all(source);

    assert_eq!(tokens.len(), 3);

    assert_eq!(tokens[0].span.start.column, 1);
    assert_eq!(tokens[0].span.end.column, 2);

    assert_eq!(tokens[1].span.start.column, 3);
    assert_eq!(tokens[1].span.end.column, 4);

    assert_eq!(tokens[2].span.start.column, 5);
    assert_eq!(tokens[2].span.end.column, 6);
}

#[test]
fn test_single_char_tokens() {
    let source = "();:";
    let tokens = lex_all(source);

    assert_eq!(tokens.len(), 4);
    assert_eq!(tokens[0].token_type, TokenType::LeftParen);
    assert_eq!(tokens[1].token_type, TokenType::RightParen);
    assert_eq!(tokens[2].token_type, TokenType::Semicolon);
    assert_eq!(tokens[3].token_type, TokenType::Colon);
}

#[test]
fn test_skip_whitespace() {
    let source = "   \t\n  let";
    let tokens = lex_all(source);

    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].token_type, TokenType::Keyword("let".into()));
}

#[test]
fn test_valid_decimal_number() {
    let source = "123.45";
    let tokens = lex_all(source);

    match &tokens[0].token_type {
        TokenType::Number(num) => assert_eq!(num, "123.45"),
        _ => panic!("Expected number token"),
    }
}

#[test]
fn test_invalid_decimal_number() {
    let source = "123.";
    let tokens = lex_all(source);

    assert!(matches!(&tokens[0].token_type, TokenType::Error(_)));
    assert!(
        tokens[0]
            .errors
            .iter()
            .any(|e| matches!(e.kind, ErrorKind::InvalidDecimal))
    );
}

#[test]
fn test_keywords() {
    let keywords = vec![
        "let", "var", "fn", "rec", "type", "if", "else", "return", "class",
    ];

    for kw in keywords {
        let tokens = lex_all(kw);

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token_type, TokenType::Keyword(kw.to_string()));
    }
}

#[test]
fn test_valid_operators() {
    let operators = vec![
        ("==", TokenType::EqualEqual),
        ("!=", TokenType::NotEqual),
        ("<=", TokenType::LessEqual),
        (">=", TokenType::GreaterEqual),
        ("+=", TokenType::PlusEqual),
        ("-=", TokenType::MinusEqual),
        ("->", TokenType::Arrow),
        ("&&", TokenType::And),
        ("||", TokenType::Or),
        ("!", TokenType::Bang),
        ("<", TokenType::Less),
        (">", TokenType::Greater),
        ("+", TokenType::Plus),
        ("-", TokenType::Minus),
        ("*", TokenType::Star),
        ("/", TokenType::Slash),
        ("%", TokenType::Modulo),
        ("&", TokenType::BitwiseAnd),
        ("|", TokenType::BitwiseOr),
        ("^", TokenType::BitwiseXor),
        ("=", TokenType::Equal),
    ];

    for (op, expected) in operators {
        let tokens = lex_all(op);

        assert_eq!(
            tokens[0].token_type, expected,
            "Failed for operator: {}",
            op
        );
    }
}

#[test]
fn test_unterminated_escape_sequence() {
    let source = r#""\"#;
    let tokens = lex_all(source);

    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].errors.len(), 2);

    let error_kinds: Vec<_> = tokens[0].errors.iter().map(|e| &e.kind).collect();

    assert!(matches!(
        error_kinds[0],
        ErrorKind::UnterminatedEscapeSequence
    ));

    assert!(matches!(error_kinds[1], ErrorKind::UnterminatedString));
}

#[test]
fn test_single_backslash_in_string() {
    let source = r#""\"#;
    let tokens = lex_all(source);

    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].errors.len(), 2);
    assert!(
        tokens[0]
            .errors
            .iter()
            .any(|e| matches!(e.kind, ErrorKind::UnterminatedEscapeSequence))
    );

    assert!(
        tokens[0]
            .errors
            .iter()
            .any(|e| matches!(e.kind, ErrorKind::UnterminatedString))
    );
}

#[test]
fn test_valid_escape_sequences() {
    let source = r#""\n\t\r\\\"a""#;
    let tokens = lex_all(source);

    assert_eq!(tokens.len(), 1);

    match &tokens[0].token_type {
        TokenType::String(s) => assert_eq!(s, "\n\t\r\\\"a"),

        _ => panic!("Expected string token"),
    }
}

#[test]
fn test_invalid_escape() {
    let source = r#""\x""#;
    let tokens = lex_all(source);

    assert!(
        tokens[0]
            .errors
            .iter()
            .any(|e| matches!(e.kind, ErrorKind::InvalidEscape('x')))
    );
}

#[test]
fn test_unexpected_character() {
    let source = "@";
    let tokens = lex_all(source);

    assert!(matches!(&tokens[0].token_type, TokenType::Error(_)));
    assert!(
        tokens[0]
            .errors
            .iter()
            .any(|e| matches!(e.kind, ErrorKind::UnexpectedCharacter('@')))
    );
}

#[test]
fn test_error_kind_messages() {
    let test_cases = vec![
        (
            ErrorKind::UnexpectedCharacter('@'),
            "E000",
            "Unexpected character: '@'",
            "Unexpected character",
        ),
        (
            ErrorKind::InvalidDecimal,
            "E001",
            "Invalid decimal number",
            "Expected digits after decimal point",
        ),
        (
            ErrorKind::InvalidOperator("??".into()),
            "E002",
            "Invalid operator: ??",
            "Invalid operator",
        ),
        (
            ErrorKind::InvalidEscape('x'),
            "E003",
            "Invalid escape sequence: \\x",
            "Invalid escape sequence",
        ),
        (
            ErrorKind::UnterminatedString,
            "E004",
            "Unterminated string literal",
            "Unterminated string",
        ),
        (
            ErrorKind::UnterminatedEscapeSequence,
            "E006",
            "Unterminated escape sequence",
            "Unterminated escape sequence",
        ),
    ];

    for (error, code, message, label) in test_cases {
        assert_eq!(error.code(), code);
        assert_eq!(error.message(), message);
        assert_eq!(error.label(), label);
    }
}
