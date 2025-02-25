use lalrpop_util::lalrpop_mod;

lalrpop_mod!(grammar);

pub fn parse(source: &str) -> Result<i32, Box<dyn std::error::Error + '_>> {
    let a = grammar::FructoseScriptParser::new().parse(source)?;
    Ok(a)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = parse("0");
        assert_eq!(result.unwrap(), 0);
    }
}
