mod expression;

use anyhow::bail;
use crate::compiler::lexer::{Lexer, Token};
use crate::compiler::parser::Statement::NoMoreStatements;

#[macro_export]
#[doc(hidden)]
macro_rules! excess_token {
    ($expr:expr) => {
        bail!("excess token: {token:?}", token = $expr)
    };
    (expected $($enum_kinds:ident )|+) => {
        bail!("Expected: {}", stringify!($($enum_kinds |)+))
    };
    (expected $ex:tt, actual $ac:expr) => {
        bail!("Expected: {}, Actual: {:?}", stringify!($ex), $ac)
    };
    (expected $ex:tt, actual $ac:tt) => {
        bail!("Expected: {}, Actual: {}", stringify!($ex), stringify!($ac))
    };
}

struct Parser {
    lexer: Lexer
}

impl Parser {
    #[allow(dead_code)]
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

#[allow(dead_code)]
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

    //noinspection RsLiveness
    fn read(parser: &Parser) -> Result<Self, Self::Err> {
        match parser.lexer.peek() {
            Token::Identifier { inner } => {
                parser.lexer.next();
                Ok(Identifier(inner))
            }
            other => excess_token!(other),
        }
    }
}

#[allow(dead_code)]
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
                    other => excess_token!(expected Identifier, actual other),
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
                excess_token!(other_token)
            }
        }
    }
}

struct UnresolvedTypeName(IdentifierOrMemberPath);

impl FromParser for UnresolvedTypeName {
    type Err = <IdentifierOrMemberPath as FromParser>::Err;

    fn read(parser: &Parser) -> Result<Self, Self::Err> {
        parser.parse().map(Self)
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
            excess_token!(expected Identifier | MemberPath)
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
                    excess_token!(expected identifier, actual other);
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
