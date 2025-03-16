use exx::{lexer::Lexer, token::Token, tokentype::TokenType};

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
            assert!(msg.contains("String literal error"));
            assert_eq!(tokens[0].span.start.absolute, 0);
            assert_eq!(tokens[0].span.end.absolute, source.len());
        }

        _ => panic!("Expected error token for unterminated string"),
    }
}
