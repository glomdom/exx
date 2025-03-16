use crate::lexer::Lexer;
use ariadne::{Color, Label, Report, ReportKind, Source};
use parser::{Parser, ParserToken};

mod ast;
mod lexer;
mod parser;
mod position;
mod span;
mod token;
mod tokentype;

fn main() {
    let source = r#"class Person {
    let name: string;
    let age: number;

    fn new(name: string, age: number) -> Person {
        return Person { name, age };
    }
}"#;
    let lexer = Lexer::new(source);

    let error_color = Color::Fixed(81);
    let tokens: Vec<_> = lexer.collect();

    println!("{}", source);

    for token in &tokens {
        if !token.errors.is_empty() {
            for diag in &token.errors {
                Report::build(
                    ReportKind::Error,
                    (
                        "anonymous",
                        diag.span.start.absolute..diag.span.end.absolute,
                    ),
                )
                .with_code(diag.kind.code())
                .with_message(diag.kind.message())
                .with_label(
                    Label::new((
                        "anonymous",
                        diag.span.start.absolute..diag.span.end.absolute,
                    ))
                    .with_message(diag.kind.label())
                    .with_color(error_color),
                )
                .finish()
                .print(("anonymous", Source::from(source)))
                .unwrap();
            }
        } else {
            println!(
                "{:?} @ {}..{} (line {}, col {}:{})",
                token.token_type,
                token.span.start.absolute,
                token.span.end.absolute,
                token.span.start.line,
                token.span.start.column,
                token.span.end.column
            );
        }
    }

    let parser_tokens: Vec<ParserToken> = tokens.into_iter().map(|t| t.into()).collect();

    dbg!(&parser_tokens);

    let mut parser = Parser::new(parser_tokens);
    match parser.parse_program() {
        Ok(expr) => {
            dbg!(expr);
        }

        Err(err) => {
            println!("{}", err.message);
        }
    }
}
