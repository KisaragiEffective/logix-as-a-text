pub mod expression;

use anyhow::bail;
use crate::compiler::lexer::{Lexer, Token};
use crate::compiler::parser::Statement::NoMoreStatements;

pub struct Parser {
    lexer: Lexer
}

impl Parser {
    pub(crate) fn with_lexer(lexer: Lexer) -> Self {
        Self {
            lexer
        }
    }

    pub(crate) fn parse<T: FromParser>(&self) -> Result<T, T::Err> {
        T::read(self)
    }
}

pub(in self) trait FromParser: Sized {
    type Err;
    
    fn read(parser: &Parser) -> Result<Self, Self::Err>;
}

pub(crate) struct RootAst {
    pub(crate) commands: Vec<Statement>,
}

impl FromParser for RootAst {
    type Err = ();

    fn read(parser: &Parser) -> Result<Self, Self::Err> {
        let mut vec = vec![];

        while let Ok(parsed_statement) = parser.parse() {
            vec.push(parsed_statement);
        }

        Ok(Self {
            commands: vec
        })
    }
}

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
pub(crate) struct Identifier(pub(crate) String);

impl FromParser for Identifier {
    type Err = anyhow::Error;

    fn read(parser: &Parser) -> Result<Self, Self::Err> {
        match parser.lexer.peek() {
            Token::Identifier { inner } => {
                parser.lexer.next();
                Ok(Identifier(inner))
            }
            other => bail!("{other:?} is unexpected, identifier was expected"),
        }
    }
}

pub enum Statement {
    NodeDeclaration {
        identifier: Identifier,
        type_tag: Option<UnresolvedTypeName>,
        rhs: RightHandSideValue,
    },
    Comment {
        content: String,
    },
    NoMoreStatements,
}

impl FromParser for Statement {
    type Err = anyhow::Error;

    fn read(parser: &Parser) -> Result<Self, Self::Err> {
        match parser.lexer.peek() {
            Token::VarKeyword => {
                parser.lexer.next();
                let ident = match parser.lexer.next() {
                    Token::Identifier { inner } => inner,
                    _ => bail!("Identifier expected")
                };

                let type_tag = if parser.lexer.peek() == Token::SymColon {
                    parser.lexer.next();
                    let type_tag = parser.parse::<UnresolvedTypeName>()?;
                    Some(type_tag)
                } else {
                    None
                };

                assert_eq!(parser.lexer.next(), Token::SymEq, "SymEq expected");
                let node = parser.parse::<RightHandSideValue>()?;

                Ok(Self::NodeDeclaration {
                    identifier: Identifier(ident),
                    type_tag,
                    rhs: node,
                })
            }
            Token::EndOfFile => {
                Ok(NoMoreStatements)
            }
            other_token => {
                bail!("Unexpected token: {other_token:?}");
            }
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct UnresolvedTypeName(pub MemberPath);

impl FromParser for UnresolvedTypeName {
    type Err = <RightHandSideValue as FromParser>::Err;

    fn read(parser: &Parser) -> Result<Self, Self::Err> {
        parser.parse().map(|a| Self(a))
    }
}

pub enum RightHandSideValue {
    Identifier(Identifier),
    MemberPath(MemberPath),
    Expression(self::expression::LogicalOrExpression),
}

impl FromParser for RightHandSideValue {
    type Err = anyhow::Error;

    fn read(parser: &Parser) -> Result<Self, Self::Err> {
        if let Ok(identifier) = parser.parse() {
            Ok(Self::Identifier(identifier))
        } else if let Ok(member_path) = parser.parse() {
            Ok(Self::MemberPath(member_path))
        } else if let Ok(first) = parser.parse() {
            Ok(Self::Expression(first))
        } else {
            bail!("expected member_path or identifier")
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct MemberPath {
    pub pack: Vec<Identifier>,
}

impl FromParser for MemberPath {
    type Err = anyhow::Error;

    fn read(parser: &Parser) -> Result<Self, Self::Err> {
        let mut buf = vec![];
        loop {
            match parser.lexer.peek() {
                Token::Identifier { inner } => {
                    parser.lexer.next();
                    buf.push(Identifier(inner))
                }
                other => {
                    bail!("{other:?} was not expected, identifier was expected")
                }
            }

            match parser.lexer.peek() {
                Token::SymDot => {
                    parser.lexer.next();
                }
                _ => break,
            }
        }

        Ok(Self {
            pack: buf
        })
    }
}
