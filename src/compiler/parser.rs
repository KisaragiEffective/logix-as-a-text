mod expression;

use anyhow::bail;
use crate::compiler::lexer::{Lexer, Token};
use crate::compiler::parser::Statement::NoMoreStatements;

struct Parser {
    lexer: Lexer
}

impl Parser {
    fn with_lexer(lexer: Lexer) -> Self {
        Self {
            lexer
        }
    }

    fn parse<T: FromParser>(&self) -> Result<T, T::Err> {
        T::read(self)
    }
}

pub(in self) trait FromParser: Sized {
    type Err;
    
    fn read(parser: &Parser) -> Result<Self, Self::Err>;
}

struct RootAst {
    commands: Vec<Statement>,
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

struct Identifier(String);

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
enum Statement {
    NodeDeclaration {
        identifier: Identifier,
        type_tag: Option<UnresolvedTypeName>,
        rhs: IdentifierOrMemberPath,
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
                let node = parser.parse::<IdentifierOrMemberPath>()?;

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

struct UnresolvedTypeName(IdentifierOrMemberPath);

impl FromParser for UnresolvedTypeName {
    type Err = <IdentifierOrMemberPath as FromParser>::Err;

    fn read(parser: &Parser) -> Result<Self, Self::Err> {
        parser.parse().map(|a| Self(a))
    }
}

enum IdentifierOrMemberPath {
    Identifier(Identifier),
    MemberPath(MemberPath),
}

impl FromParser for IdentifierOrMemberPath {
    type Err = anyhow::Error;

    fn read(parser: &Parser) -> Result<Self, Self::Err> {
        if let Ok(identifier) = parser.parse() {
            Ok(Self::Identifier(identifier))
        } else if let Ok(member_path) = parser.parse() {
            Ok(Self::MemberPath(member_path))
        } else {
            bail!("expected member_path or identifier")
        }
    }
}

struct MemberPath {
    pack: Vec<Identifier>,
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
