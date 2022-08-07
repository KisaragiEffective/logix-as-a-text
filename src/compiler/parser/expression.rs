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

enum Cast {
    Do {
        lhs: Box<Self>,
        tp: UnresolvedTypeName,
    },
    Propagated(First),
}

// ------------------------------------------------

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

// ------------------------------------------------

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
