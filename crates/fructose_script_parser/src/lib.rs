pub mod ast;
mod visit;

use lalrpop_util::{ParseError, lalrpop_mod, lexer};

lalrpop_mod!(grammar);

pub use grammar::FructoseScriptParser;
pub use visit::Visit;

pub type FructoseParseError<'a> = ParseError<usize, lexer::Token<'a>, &'static str>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn let_statement() {
        let result = FructoseScriptParser::new().parse("let a = 0;");
        let stmt = ast::Statement::from(ast::Let {
            name: ast::Ident {
                value: String::from("a"),
                range: 4..5,
            },
            init: ast::NatLiteral {
                value: 0,
                range: 8..9,
            }
            .into(),
            range: 0..10,
        });
        let module = ast::Module {
            items: vec![stmt.into()],
        };
        assert_eq!(result.unwrap(), module);
    }

    #[test]
    fn assign_statement() {
        let result = FructoseScriptParser::new().parse("a = 0;");
        let stmt = ast::Statement::from(ast::Assign {
            target: ast::Ident {
                value: String::from("a"),
                range: 0..1,
            },
            value: ast::NatLiteral {
                value: 0,
                range: 4..5,
            }
            .into(),
            range: 0..6,
        });
        let module = ast::Module {
            items: vec![stmt.into()],
        };
        assert_eq!(result.unwrap(), module);
    }

    #[test]
    fn block_expression() {
        let result = FructoseScriptParser::new().parse("{ let a = 0; a };");
        let expr = ast::Expression::from(ast::Block {
            statements: vec![ast::Statement::from(ast::Let {
                name: ast::Ident {
                    value: String::from("a"),
                    range: 6..7,
                },
                init: ast::NatLiteral {
                    value: 0,
                    range: 10..11,
                }
                .into(),
                range: 2..12,
            })],
            last: Some(Box::new(
                ast::Ident {
                    value: String::from("a"),
                    range: 13..14,
                }
                .into(),
            )),
            range: 0..16,
        });
        let module = ast::Module {
            items: vec![ast::Statement::from(expr).into()],
        };
        assert_eq!(result.unwrap(), module);
    }
}
