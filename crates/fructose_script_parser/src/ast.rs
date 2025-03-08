use std::ops::Range;

macro_rules! enum_ {
    ($(#[$attr:meta])* $vis:vis enum $name:ident [ $($xs:ident),+ ]) => {
        $(#[$attr])* $vis enum $name {
            $($xs($xs)),+
        }

        $(impl From<$xs> for $name {
            #[inline]
            fn from(value: $xs) -> Self {
                $name::$xs(value)
            }
        })+
    };
}

#[derive(Debug, PartialEq)]
pub struct Module {
    pub items: Vec<ModuleItem>,
}

enum_!(
    #[derive(Debug, PartialEq)]
    pub enum ModuleItem [Statement]
);

enum_!(
    #[derive(Debug, PartialEq)]
    pub enum Statement [Let, Assign, Expression]
);

#[derive(Debug, PartialEq)]
pub struct Let {
    pub name: Ident,
    pub init: Expression,
    pub range: Range<usize>,
}

#[derive(Debug, PartialEq)]
pub struct Assign {
    pub target: Ident,
    pub value: Expression,
    pub range: Range<usize>,
}

enum_!(
    #[derive(Debug, PartialEq)]
    pub enum Expression [Ident, NatLiteral, Block, Fn]
);

#[derive(Debug, PartialEq)]
pub struct Ident {
    pub value: String,
    pub range: Range<usize>,
}

#[derive(Debug, PartialEq)]
pub struct NatLiteral {
    pub value: u32,
    pub range: Range<usize>,
}

#[derive(Debug, PartialEq)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub last: Option<Box<Expression>>,
    pub range: Range<usize>,
}

#[derive(Debug, PartialEq)]
pub struct Fn {
    pub parameters: Vec<Ident>,
    pub body: Box<Expression>,
    pub range: Range<usize>,
}
