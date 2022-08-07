use anyhow::bail;
use crate::compiler::lexer::Token;
use crate::compiler::parser::{FromParser, Identifier, Parser, UnresolvedTypeName};

trait BinaryOperatorNode {
    type OperatorEnum;
    type Rhs;

    fn binary(operator: Self::OperatorEnum, lhs: Self, rhs: Self::Rhs) -> Self;
}

trait PropagateFrom<From> {
    fn propagate(from: From) -> Self;
}

// ------------------------------------------------

enum First {
    IntegralLiteral {
        sequence: String,
    },
    StringLiteral {
        sequence: String,
    },
    Variable {
        identifier: Identifier,
    },
    True,
    False,
}

impl FromParser for First {
    type Err = anyhow::Error;

    fn read(parser: &Parser) -> Result<Self, Self::Err> {
        match parser.lexer.peek() {
            Token::Identifier { inner } => {
                parser.lexer.next();
                let identifier = Identifier(inner);
                let var_node = First::Variable {
                    identifier
                };

                Ok(var_node)
            }
            Token::Digits { sequence } => {
                Ok(Self::IntegralLiteral {
                    sequence
                })
            }
            Token::StringLiteral { content } => {
                Ok(Self::StringLiteral { sequence: content })
            }
            Token::KeywordTrue => {
                Ok(Self::True)
            }
            Token::KeywordFalse => {
                Ok(Self::False)
            }
            other => {
                bail!("unexpected token: {other:?}")
            }
        }
    }
}
// ------------------------------------------------

/// left-associative
/// e.g. `1 as u16 as u32` is equivalent with `(1 as u16) as u32`.
enum Cast {
    Do {
        lhs: Box<Self>,
        tp: UnresolvedTypeName,
    },
    Propagated(First),
}

impl FromParser for Cast {
    type Err = anyhow::Error;

    fn read(parser: &Parser) -> Result<Self, Self::Err> {
        let first_term = parser.parse::<Cast>()?;
        if parser.lexer.peek() == Token::KeywprdAs {
            parser.lexer.next();
            let type_name = parser.parse()?;
            Ok(Self::Do {
                lhs: Box::new(first_term),
                tp: type_name
            })
        } else {
            Ok(first_term)
        }
    }
}

// ------------------------------------------------

/// left-associative
enum Multiplicative {
    Binary {
        operator: <Self as BinaryOperatorNode>::OperatorEnum,
        lhs: Box<Self>,
        rhs: Box<Self>,
    },
    Propagated(Cast)
}

enum MultiplicativeOps {
    /// `*`
    Multiply,
    /// `/`
    Divide,
    /// `%`
    Reminder,
}

impl BinaryOperatorNode for Multiplicative {
    type OperatorEnum = MultiplicativeOps;
    type Rhs = Self;

    fn binary(operator: Self::OperatorEnum, lhs: Self, rhs: Self) -> Self {
        Self::Binary {
            operator,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }
    }
}

impl FromParser for Multiplicative {
    type Err = anyhow::Error;

    fn read(parser: &Parser) -> Result<Self, Self::Err> {
        let first_term = parser.parse::<Cast>()?;
        let next_token = parser.lexer.peek();
        let asterisk_or_slash = |token: &Token| {
            token == &Token::SymAsterisk || token == &Token::SymSlash
        };

        if asterisk_or_slash(&next_token) {
            // SymAsterisk | SymSlash
            parser.lexer.next();
            let operator_token = next_token;
            let lhs = Self::Propagated(first_term);
            let rhs = parser.parse()?;
            let get_operator_from_token = |token: &Token| {
                match token {
                    Token::SymAsterisk => MultiplicativeOps::Multiply,
                    Token::SymSlash => MultiplicativeOps::Divide,
                    e => panic!("excess token: {e:?}")
                }
            };

            let mut acc = Self::binary(get_operator_from_token(&operator_token), lhs, Self::Propagated(rhs));
            let mut operator_token = parser.lexer.peek();
            while asterisk_or_slash(&operator_token) {
                // SymAsterisk | SymSlash
                parser.lexer.next();
                let new_rhs = Self::Propagated(parser.parse()?);
                // 左結合になるように詰め替える
                // これは特に除算のときに欠かせない処理である
                acc = Self::binary(get_operator_from_token(&operator_token), acc, new_rhs);
                operator_token = parser.lexer.peek();
            }
            Ok(acc)
        } else {
            // it is unary
            Ok(Self::Propagated(first_term))
        }
    }
}

// ------------------------------------------------

/// left-associative
enum Additive {
    Binary {
        operator: <Self as BinaryOperatorNode>::OperatorEnum,
        lhs: Box<Self>,
        rhs: Box<Self>,
    },
    Propagated(Multiplicative)
}

enum AdditiveOps {
    Add,
    Subtract,
}

impl BinaryOperatorNode for Additive {
    type OperatorEnum = AdditiveOps;
    type Rhs = Self;

    fn binary(operator: Self::OperatorEnum, lhs: Self, rhs: Self) -> Self {
        Self::Binary {
            operator,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }
    }
}

impl FromParser for Additive {
    type Err = anyhow::Error;

    fn read(parser: &Parser) -> Result<Self, Self::Err> {
        let first_term = parser.parse::<Multiplicative>()?;
        let next_token = parser.lexer.peek();
        let plus_or_minus = |token: &Token| {
            token == &Token::SymPlus || token == &Token::SymMinus
        };

        if plus_or_minus(&next_token) {
            // SymPlus | SymMinus
            parser.lexer.next();
            let operator_token = next_token;
            let lhs = Self::Propagated(first_term);
            let rhs = parser.parse::<Multiplicative>()?;
            let get_operator_from_token = |token: &Token| {
                match token {
                    Token::SymPlus => AdditiveOps::Add,
                    Token::SymMinus => AdditiveOps::Subtract,
                    e => panic!("excess token: {e:?}")
                }
            };

            let mut acc = Self::binary(get_operator_from_token(&operator_token), lhs, Self::Propagated(rhs));
            let mut operator_token = parser.lexer.peek();
            while plus_or_minus(&operator_token) {
                // SymPlus | SymMinus
                parser.lexer.next();
                let new_rhs = parser.parse()?;
                // 左結合になるように詰め替える
                // これは特に減算のときに欠かせない処理である
                acc = Self::binary(get_operator_from_token(&operator_token), acc, Self::Propagated(new_rhs));
                operator_token = parser.lexer.peek();
            }
            Ok(acc)
        } else {
            // it is unary or multiplicative
            Ok(Self::Propagated(first_term))
        }
    }
}
// ------------------------------------------------

enum BitwiseShift {
    Binary {
        operator: <Self as BinaryOperatorNode>::OperatorEnum,
        lhs: Box<Self>,
        rhs: Box<Additive>,
    },
    Propagated(Additive)
}

enum BitwiseShiftOps {
    LeftShift,
    RightShift,
}

impl BinaryOperatorNode for BitwiseShift {
    type OperatorEnum = BitwiseShiftOps;
    type Rhs = Additive;

    fn binary(operator: Self::OperatorEnum, lhs: Self, rhs: Self::Rhs) -> Self {
        Self::Binary {
            operator,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs)
        }
    }
}

impl FromParser for BitwiseShift {
    type Err = ();

    fn read(parser: &Parser) -> Result<Self, Self::Err> {
        todo!()
    }
}
// ------------------------------------------------

enum RelationCheckExpression {
    Binary {
        operator: <Self as BinaryOperatorNode>::OperatorEnum,
        lhs: Box<Self>,
        rhs: Box<BitwiseShift>,
    },
    Propagated(BitwiseShift)
}

enum RelationCheckExpressionOps {
    Less,
    LessEqual,
    More,
    MoreEqual,
    Spaceship,
}

impl BinaryOperatorNode for RelationCheckExpression {
    type OperatorEnum = RelationCheckExpressionOps;
    type Rhs = BitwiseShift;

    fn binary(operator: Self::OperatorEnum, lhs: Self, rhs: Self::Rhs) -> Self {
        Self::Binary {
            operator,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs)
        }
    }
}

// ------------------------------------------------

enum EqualityCheckExpression {
    Binary {
        operator: <Self as BinaryOperatorNode>::OperatorEnum,
        lhs: Box<Self>,
        rhs: Box<RelationCheckExpression>,
    },
    Propagated(RelationCheckExpression)
}

enum EqualityCheckExpressionOps {
    Equal,
    NotEqual,
}

impl BinaryOperatorNode for EqualityCheckExpression {
    type OperatorEnum = EqualityCheckExpressionOps;
    type Rhs = RelationCheckExpression;

    fn binary(operator: Self::OperatorEnum, lhs: Self, rhs: Self::Rhs) -> Self {
        Self::Binary {
            operator,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs)
        }
    }
}

// ------------------------------------------------

enum BitwiseAndExpression {
    Binary {
        operator: <Self as BinaryOperatorNode>::OperatorEnum,
        lhs: Box<Self>,
        rhs: Box<EqualityCheckExpression>,
    },
    Propagated(EqualityCheckExpression)
}

enum BitwiseAndExpressionOp {
    BitwiseAnd,
}

impl BinaryOperatorNode for BitwiseAndExpression {
    type OperatorEnum = BitwiseAndExpressionOp;
    type Rhs = EqualityCheckExpression;

    fn binary(operator: Self::OperatorEnum, lhs: Self, rhs: Self::Rhs) -> Self {
        Self::Binary {
            operator,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs)
        }
    }
}

// ------------------------------------------------

enum BitwiseXorExpression {
    Binary {
        operator: <Self as BinaryOperatorNode>::OperatorEnum,
        lhs: Box<Self>,
        rhs: Box<BitwiseAndExpression>,
    },
    Propagated(BitwiseAndExpression)
}

enum BitwiseXorExpressionOp {
    BitwiseXor
}

impl BinaryOperatorNode for BitwiseXorExpression {
    type OperatorEnum = BitwiseXorExpressionOp;
    type Rhs = BitwiseAndExpression;

    fn binary(operator: Self::OperatorEnum, lhs: Self, rhs: Self::Rhs) -> Self {
        Self::Binary {
            operator,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs)
        }
    }
}

// ------------------------------------------------

enum BitwiseOrExpression {
    Binary {
        operator: <Self as BinaryOperatorNode>::OperatorEnum,
        lhs: Box<Self>,
        rhs: Box<BitwiseXorExpression>,
    },
    Propagated(BitwiseXorExpression)
}

enum BitwiseOrExpressionOp {
    BitwiseOr,
}

impl BinaryOperatorNode for BitwiseOrExpression {
    type OperatorEnum = BitwiseOrExpressionOp;
    type Rhs = BitwiseXorExpression;

    fn binary(operator: Self::OperatorEnum, lhs: Self, rhs: Self::Rhs) -> Self {
        Self::Binary {
            operator,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs)
        }
    }
}

// ------------------------------------------------

enum LogicalAndExpression {
    Binary {
        operator: <Self as BinaryOperatorNode>::OperatorEnum,
        lhs: Box<Self>,
        rhs: Box<LogicalOrExpression>,
    },
    Propagated(BitwiseOrExpression),
}

enum LogicalAndExpressionOp {
    LogicalAnd
}

impl BinaryOperatorNode for LogicalAndExpression {
    type OperatorEnum = LogicalAndExpressionOp;
    type Rhs = LogicalOrExpression;

    fn binary(operator: Self::OperatorEnum, lhs: Self, rhs: Self::Rhs) -> Self {
        Self::Binary {
            operator,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs)
        }
    }
}

// ------------------------------------------------

enum LogicalOrExpression {
    Binary {
        operator: <Self as BinaryOperatorNode>::OperatorEnum,
        lhs: Box<Self>,
        rhs: Box<LogicalAndExpression>,
    },
    Propagated(LogicalAndExpression),
}

enum LogicalOrExpressionOp {
    LogicalOr
}

impl BinaryOperatorNode for LogicalOrExpression {
    type OperatorEnum = LogicalOrExpressionOp;
    type Rhs = LogicalAndExpression;

    fn binary(operator: Self::OperatorEnum, lhs: Self, rhs: Self::Rhs) -> Self {
        Self::Binary {
            operator,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }
    }
}
