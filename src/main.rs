use ariadne::{ColorGenerator, Label, Report, ReportKind, Source};
use tokentype::TokenType;

use crate::lexer::Lexer;

mod lexer;
mod position;
mod span;
mod token;
mod tokentype;

fn main() {
    let source = "1 + 1\n\"aaaaa";
    let lexer = Lexer::new(source);

    let mut colors = ColorGenerator::new();

    let a = colors.next();

    println!("{}", source);

    for token in lexer {
        match token.token_type {
            TokenType::Error(ref err_msg) => {
                dbg!(&token);

                Report::build(
                    ReportKind::Error,
                    (
                        "anonymous",
                        (token.span.start.absolute..token.span.end.absolute),
                    ),
                )
                .with_code(1)
                .with_message(format!("{}", err_msg))
                .with_label(
                    Label::new((
                        "anonymous",
                        (token.span.start.absolute..token.span.end.absolute + 1),
                    ))
                    .with_message("This escape is invalid")
                    .with_color(a),
                )
                .with_note("ngl u should give up coding")
                .finish()
                .print(("anonymous", Source::from(source)))
                .unwrap();

                break;
            }

            _ => println!(
                "{:?} @ {}..{} (line {}, col {}-{})",
                token.token_type,
                token.span.start.absolute,
                token.span.end.absolute,
                token.span.start.line,
                token.span.start.column,
                token.span.end.column
            ),
        }
    }
}
