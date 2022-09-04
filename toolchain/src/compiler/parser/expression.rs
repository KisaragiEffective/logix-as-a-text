use anyhow::bail;
use crate::compiler::lexer::Token;
use crate::compiler::parser::{FromParser, Identifier, Parser, UnresolvedTypeName};

trait BinaryOperatorNode {
    type OperatorEnum: Copy + FromParser;
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
        if parser.lexer.peek() == Token::KeywordAs {
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

macro_rules! binary_expression_node {
    ($name:ident, assoc: left, derive: $propagate_from:ident, rhs: $rhs:ty, operator: $operators:ty) => {
        #[doc="left-associative"]
        enum $name {
            // We'll handle them in the future
            #[allow(dead_code)]
            Binary {
                operator: <Self as BinaryOperatorNode>::OperatorEnum,
                lhs: Box<Self>,
                rhs: Box<$rhs>,
            },
            Propagated($propagate_from)
        }
        binary_expression_node_0!($name, derive: $propagate_from, rhs: $rhs, operator: $operators);
    };
    ($name:ident, assoc: right, derive: $propagate_from:ident, rhs: $rhs:ty, operator: $operators:ty) => {
        #[doc="right-associative"]
        enum $name {
            // We'll handle them in the future
            #[allow(dead_code)]
            Binary {
                operator: <Self as BinaryOperatorNode>::OperatorEnum,
                lhs: Box<Self>,
                rhs: Box<$rhs>,
            },
            Propagated($propagate_from)
        }
        binary_expression_node_0!($name, derive: $propagate_from, rhs: $rhs, operator: $operators);
    };
}

macro_rules! binary_expression_node_0 {
    ($name:ident, derive: $propagate_from:ident, rhs: $rhs:ty, operator: $operators:ty) => {
        impl PropagateFrom<$propagate_from> for $name {
            fn propagate(from: $propagate_from) -> Self {
                Self::Propagated(from)
            }
        }

        impl BinaryOperatorNode for $name {
            type OperatorEnum = $operators;
            type Rhs = $rhs;
            fn binary(operator: Self::OperatorEnum, lhs: Self, rhs: $rhs) -> Self {
                Self::Binary {
                    operator,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                }
            }
        }
    };
}

macro_rules! parse_binary_expression_node {
    (left_assoc, $name:ty) => {
        impl FromParser for $name {
            type Err = anyhow::Error;

            fn read(parser: &Parser) -> Result<Self, Self::Err> {
                let first_term = parser.parse()?;
                let operator = <<Self as BinaryOperatorNode>::OperatorEnum as FromParser>::read(parser);

                if let Ok(operator_token) = operator {
                    parser.lexer.next();
                    let lhs = Self::Propagated(first_term);
                    let rhs = parser.parse()?;

                    let mut acc = Self::binary(operator_token, lhs, rhs);
                    let next = <<Self as BinaryOperatorNode>::OperatorEnum as FromParser>::read(parser);
                    while let Ok(op) = next {
                        let new_rhs = parser.parse()?;
                        // 左結合になるように詰め替える
                        acc = Self::binary(op, acc, new_rhs);
                    }
                    Ok(acc)
                } else {
                    Ok(Self::Propagated(first_term))
                }
            }
        }
    };
    (right_assoc, $name:ty) => {
                impl FromParser for $name {
            type Err = anyhow::Error;

            fn read(parser: &Parser) -> Result<Self, Self::Err> {
                let first_term = parser.parse()?;
                let operator = <Self as BinaryOperatorNode>::OperatorEnum::read(parser);

                if let Ok(operator_token) = operator {
                    parser.lexer.next();
                    let lhs = Self::Propagated(first_term);
                    let rhs = parser.parse()?;

                    let mut acc = Self::binary(operator_token, lhs, Self::Propagated(rhs));
                    Ok(acc)
                } else {
                    Ok(Self::Propagated(first_term))
                }
            }
        }
    }
}

/// https://users.rust-lang.org/t/80779/2
macro_rules! operator_from_parser {
    ($name:ty, $($token:ident => $variant:ident),+) => {
        impl FromParser for $name {
            type Err = anyhow::Error;

            fn read(parser: &Parser) -> Result<Self, Self::Err> {
                let op = match parser.lexer.peek() {
                    $(Token::$token => Self::$variant,)+
                    other => excess_token!(other)
                };

                parser.lexer.next();
                Ok(op)
            }
        }
    }
}

macro_rules! excess_token {
    ($expr:expr) => {
        bail!("excess token: {token:?}", token = $expr)
    }
}
// ------------------------------------------------

binary_expression_node!(Multiplicative, assoc: left, derive: Cast, rhs: Self, operator: MultiplicativeOps);

#[derive(Copy, Clone)]
enum MultiplicativeOps {
    /// `*`
    Multiply,
    /// `/`
    Divide,
    /// `%`
    Reminder,
}

operator_from_parser!(MultiplicativeOps, SymAsterisk => Multiply, SymSlash => Divide, SymPlus => Reminder);

parse_binary_expression_node!(left_assoc, Multiplicative);

// ------------------------------------------------

binary_expression_node!(Additive, assoc: left, derive: Multiplicative, rhs: Self, operator: AdditiveOps);

#[derive(Copy, Clone)]
enum AdditiveOps {
    Add,
    Subtract,
}

operator_from_parser!(AdditiveOps, SymPlus => Add, SymMinus => Subtract);

parse_binary_expression_node!(left_assoc, Additive);
// ------------------------------------------------

binary_expression_node!(BitwiseShift, assoc: left, derive: Additive, rhs: Additive, operator: BitwiseShiftOps);

#[derive(Copy, Clone)]
enum BitwiseShiftOps {
    LeftShift,
    RightShift,
}

operator_from_parser!(BitwiseShiftOps, PartMoreMore => LeftShift, PartLessLess => RightShift);

parse_binary_expression_node!(left_assoc, BitwiseShift);
// ------------------------------------------------

binary_expression_node!(RelationCheckExpression, assoc: left, derive: BitwiseShift, rhs: BitwiseShift, operator: RelationCheckExpressionOps);

#[derive(Copy, Clone)]
enum RelationCheckExpressionOps {
    Less,
    LessEqual,
    More,
    MoreEqual,
    Spaceship,
}

operator_from_parser!(RelationCheckExpressionOps, SymLess => Less, PartLessEq => LessEqual, SymMore => More, PartMoreEq => MoreEqual, PartLessEqMore => Spaceship);

parse_binary_expression_node!(left_assoc, RelationCheckExpression);

// ------------------------------------------------

binary_expression_node!(EqualityCheckExpression, assoc: left, derive: RelationCheckExpression, rhs: RelationCheckExpression, operator: EqualityCheckExpressionOps);

#[derive(Copy, Clone)]
enum EqualityCheckExpressionOps {
    Equal,
    NotEqual,
}

operator_from_parser!(EqualityCheckExpressionOps, PartEqEq => Equal, PartBangEq => NotEqual);

parse_binary_expression_node!(left_assoc, EqualityCheckExpression);

// ------------------------------------------------

binary_expression_node!(BitwiseAndExpression, assoc: left, derive: EqualityCheckExpression, rhs: EqualityCheckExpression, operator: BitwiseAndExpressionOp);

#[derive(Copy, Clone)]
enum BitwiseAndExpressionOp {
    BitwiseAnd,
}

operator_from_parser!(BitwiseAndExpressionOp, SymAnd => BitwiseAnd);

parse_binary_expression_node!(left_assoc, BitwiseAndExpression);
// ------------------------------------------------

binary_expression_node!(BitwiseXorExpression, assoc: left, derive: BitwiseAndExpression, rhs: BitwiseAndExpression, operator: BitwiseXorExpressionOp);

#[derive(Copy, Clone)]
enum BitwiseXorExpressionOp {
    BitwiseXor
}

operator_from_parser!(BitwiseXorExpressionOp, SymCaret => BitwiseXor);

parse_binary_expression_node!(left_assoc, BitwiseXorExpression);

// ------------------------------------------------

binary_expression_node!(BitwiseOrExpression, assoc: left, derive: BitwiseXorExpression, rhs: BitwiseXorExpression, operator: BitwiseOrExpressionOp);

#[derive(Copy, Clone)]
enum BitwiseOrExpressionOp {
    BitwiseOr,
}

operator_from_parser!(BitwiseOrExpressionOp, SymPipe => BitwiseOr);

parse_binary_expression_node!(left_assoc, BitwiseOrExpression);

// ------------------------------------------------

binary_expression_node!(LogicalAndExpression, assoc: left, derive: BitwiseOrExpression, rhs: BitwiseOrExpression, operator: LogicalAndExpressionOp);

#[derive(Copy, Clone)]
enum LogicalAndExpressionOp {
    LogicalAnd
}

operator_from_parser!(LogicalAndExpressionOp, PartAndAnd => LogicalAnd);

parse_binary_expression_node!(left_assoc, LogicalAndExpression);

// ------------------------------------------------

binary_expression_node!(LogicalOrExpression, assoc: left, derive: LogicalAndExpression, rhs: BitwiseAndExpression, operator: LogicalOrExpressionOp);

#[derive(Copy, Clone)]
enum LogicalOrExpressionOp {
    LogicalOr
}

operator_from_parser!(LogicalOrExpressionOp, PartPipePipe => LogicalOr);

parse_binary_expression_node!(left_assoc, LogicalOrExpression);
