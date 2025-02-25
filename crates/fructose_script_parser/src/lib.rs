mod ast;

use ast::Module;
use lalrpop_util::{ParseError, lalrpop_mod, lexer};

lalrpop_mod!(grammar);

pub fn parse(source: &str) -> Result<Module, ParseError<usize, lexer::Token<'_>, &'static str>> {
    let a = grammar::FructoseScriptParser::new().parse(source)?;
    Ok(a)
}

#[cfg(test)]
mod tests {
    use crate::ast::{Assign, Block, Expression, Ident, Let, NatLiteral, Statement};

    use super::*;

    #[test]
    fn let_statement() {
        let result = parse("let a = 0;");
        let stmt = Statement::from(Let {
            name: Ident {
                value: String::from("a"),
            },
            init: NatLiteral { value: 0 }.into(),
        });
        let module = Module {
            items: vec![stmt.into()],
        };
        assert_eq!(result.unwrap(), module);
    }

    #[test]
    fn assign_statement() {
        let result = parse("a = 0;");
        let stmt = Statement::from(Assign {
            target: Ident {
                value: String::from("a"),
            },
            value: NatLiteral { value: 0 }.into(),
        });
        let module = Module {
            items: vec![stmt.into()],
        };
        assert_eq!(result.unwrap(), module);
    }

    #[test]
    fn block_expression() {
        let result = parse("{ let a = 0; a };");
        let expr = Expression::from(Block {
            statements: vec![Statement::from(Let {
                name: Ident {
                    value: String::from("a"),
                },
                init: NatLiteral { value: 0 }.into(),
            })],
            last: Some(Box::new(
                Ident {
                    value: String::from("a"),
                }
                .into(),
            )),
        });
        let module = Module {
            items: vec![Statement::from(expr).into()],
        };
        assert_eq!(result.unwrap(), module);
    }
}
