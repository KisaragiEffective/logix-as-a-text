use std::collections::HashMap;
use anyhow::{anyhow, bail, Context};
use crate::compiler::lexer::Lexer;
use crate::compiler::parser::{Identifier, Parser, RightHandSideValue, RootAst, Statement, UnresolvedTypeName};
use crate::compiler::parser::expression::{Additive, Cast, First, LogicalOrExpression, Multiplicative, MultiplicativeOps};
use crate::test_interpreter::InterpreterError::ExecutionError;

type ExecutionResult = Result<Vec<SupportedTypeBox>, InterpreterError>;
type Tag = SupportedTypeTag;

struct TestInterpreter {
    scope: HashMap<Identifier, SupportedTypeBox>
}

impl TestInterpreter {
    fn create_and_execute(src: &str) -> ExecutionResult {
        let interpreter = TestInterpreter {
            scope: HashMap::new()
        };
        let parser = Parser::with_lexer(Lexer::create(src));
        let x = parser.parse::<RootAst>();
        match x {
            Ok(root_ast) => {
                interpreter.execute(root_ast.commands)
            }
            Err(e) => {
                Err(InterpreterError::SyntaxError)
            }
        }
    }

    fn execute(self, statements: Vec<Statement>) -> ExecutionResult {
        for statement in statements {
            let result = match statement {
                Statement::NodeDeclaration { identifier, type_tag, rhs } => {
                    let type_tag = type_tag.expect("Currently, the node declaration must have explicit type annotation.\
                    Note: This is an implementation restriction, and will be removed in future. Please see https://github.com/KisaragiEffective/logix-as-a-text/issues/6\
                          for current status.");
                    match rhs {
                        RightHandSideValue::Identifier(ident) => {
                            Err(ExecutionError(anyhow!("unsupported: identifier")))
                        }
                        RightHandSideValue::MemberPath(path) => {
                            Err(ExecutionError(anyhow!("unsupported: member_path")))
                        }
                        RightHandSideValue::Expression(expr) => {
                            expr.evaluate(&self)
                        }
                    }
                }
                Statement::Comment { .. } => {
                    // NOP
                    Ok(())
                }
                Statement::NoMoreStatements => {
                    Ok(())
                }
            };
            result?;
        }

        Ok(vec![])
    }

    pub(in self) fn resolve_dynamic(&self, t: UnresolvedTypeName) -> Option<SupportedTypeTag> {
        match t.0.pack.into_iter().map(|a| a.0).collect::<Vec<_>>().join(".").as_str() {
            "bool" => Some(SupportedTypeTag::Bool),
            "i8" => Some(SupportedTypeTag::I8),
            "u8" => Some(SupportedTypeTag::U8),
            "i16" => Some(SupportedTypeTag::I16),
            "u16" => Some(SupportedTypeTag::U16),
            "i32" => Some(SupportedTypeTag::I32),
            "u32" => Some(SupportedTypeTag::U32),
            "i64" => Some(SupportedTypeTag::I64),
            "u64" => Some(SupportedTypeTag::U64),
            "f32" => Some(SupportedTypeTag::F32),
            "f64" => Some(SupportedTypeTag::F64),
            "string" => Some(SupportedTypeTag::String),
            "impulse" => Some(SupportedTypeTag::Impulse),
            _ => None,
        }

    }
}

trait CanBeEvaluated {
    type Err;

    fn evaluate(&self, interpreter: &TestInterpreter) -> Result<SupportedTypeBox, Self::Err>;
}

impl CanBeEvaluated for First {
    type Err = anyhow::Error;

    fn evaluate(&self, interpreter: &TestInterpreter) -> Result<SupportedTypeBox, Self::Err> {
        match self {
            First::IntegralLiteral { sequence } => {
                // TODO: 実装上の都合で暗黙の型変換が起こっているがこれは規格に違反している
                Ok(SupportedTypeBox::I64(sequence.as_str().parse().context("parsing integer literal")?))
            }
            First::StringLiteral { sequence } => {
                Ok(SupportedTypeBox::String(sequence.clone()))
            }
            First::Variable { identifier } => {
                match interpreter.scope.get(identifier) {
                    None => {
                        bail!("{identifier:?} was not found")
                    }
                    Some(t) => {
                        Ok(t.clone())
                    }
                }
            }
            First::True => {
                Ok(SupportedTypeBox::Bool(true))
            }
            First::False => {
                Ok(SupportedTypeBox::Bool(false))
            }
        }
    }
}

impl CanBeEvaluated for Cast {
    type Err = anyhow::Error;

    fn evaluate(&self, interpreter: &TestInterpreter) -> Result<SupportedTypeBox, Self::Err> {
        match self {
            Cast::Do { lhs: raw_lhs, tp } => {
                let lhs = raw_lhs.evaluate(interpreter)?;
                let tt = interpreter.resolve_dynamic(tp.clone());
                if let Some(type_tag) = tt {
                    if lhs.tag() == type_tag {
                        return Ok(lhs)
                    }

                    let into = lhs.tag();

                    match into {
                        Tag::Bool => Err(anyhow!("{type_tag:?} cannot be casted to {into:?}")),
                        Tag::I8 => {
                            let lhs = match lhs {
                                SupportedTypeBox::I8(v) => v,
                                _ => unreachable!()
                            };
                            match type_tag {
                                Tag::I16 => Ok(SupportedTypeBox::I16(lhs as i16)),
                                Tag::I32 => Ok(SupportedTypeBox::I32(lhs as i32)),
                                Tag::I64 => Ok(SupportedTypeBox::I64(lhs as i64)),
                                Tag::F32 => Ok(SupportedTypeBox::F32(lhs as f32)),
                                Tag::F64 => Ok(SupportedTypeBox::F64(lhs as f64)),
                                _ => {
                                    Err(anyhow!("{type_tag:?} cannot be casted to {into:?}"))
                                }
                            }
                        }
                        Tag::U8 => {
                            let lhs = match lhs {
                                SupportedTypeBox::U8(v) => v,
                                _ => unreachable!()
                            };
                            match type_tag {
                                Tag::I16 => Ok(SupportedTypeBox::I16(lhs as i16)),
                                Tag::I32 => Ok(SupportedTypeBox::I32(lhs as i32)),
                                Tag::I64 => Ok(SupportedTypeBox::I64(lhs as i64)),
                                Tag::U16 => Ok(SupportedTypeBox::U16(lhs as u16)),
                                Tag::U32 => Ok(SupportedTypeBox::U32(lhs as u32)),
                                Tag::U64 => Ok(SupportedTypeBox::U64(lhs as u64)),
                                Tag::F32 => Ok(SupportedTypeBox::F32(lhs as f32)),
                                Tag::F64 => Ok(SupportedTypeBox::F64(lhs as f64)),
                                _ => {
                                    Err(anyhow!("{type_tag:?} cannot be casted to {into:?}"))
                                }
                            }
                        }
                        Tag::I16 => {
                            let lhs = match lhs {
                                SupportedTypeBox::I16(v) => v,
                                _ => unreachable!()
                            };
                            match type_tag {
                                Tag::I32 => Ok(SupportedTypeBox::I32(lhs as i32)),
                                Tag::I64 => Ok(SupportedTypeBox::I64(lhs as i64)),
                                Tag::F32 => Ok(SupportedTypeBox::F32(lhs as f32)),
                                Tag::F64 => Ok(SupportedTypeBox::F64(lhs as f64)),
                                _ => {
                                    Err(anyhow!("{type_tag:?} cannot be casted to {into:?}"))
                                }
                            }
                        }
                        Tag::U16 => {
                            let lhs = match lhs {
                                SupportedTypeBox::U16(v) => v,
                                _ => unreachable!()
                            };
                            match type_tag {
                                Tag::I32 => Ok(SupportedTypeBox::I32(lhs as i32)),
                                Tag::I64 => Ok(SupportedTypeBox::I64(lhs as i64)),
                                Tag::U32 => Ok(SupportedTypeBox::U32(lhs as u32)),
                                Tag::U64 => Ok(SupportedTypeBox::U64(lhs as u64)),
                                Tag::F32 => Ok(SupportedTypeBox::F32(lhs as f32)),
                                Tag::F64 => Ok(SupportedTypeBox::F64(lhs as f64)),
                                _ => {
                                    Err(anyhow!("{type_tag:?} cannot be casted to {into:?}"))
                                }
                            }
                        }
                        Tag::I32 => {
                            let lhs = match lhs {
                                SupportedTypeBox::I32(v) => v,
                                _ => unreachable!()
                            };
                            match type_tag {
                                Tag::I64 => Ok(SupportedTypeBox::I64(lhs as i64)),
                                Tag::F64 => Ok(SupportedTypeBox::F64(lhs as f64)),
                                _ => {
                                    Err(anyhow!("{type_tag:?} cannot be casted to {into:?}"))
                                }
                            }
                        }
                        Tag::U32 => {
                            let lhs = match lhs {
                                SupportedTypeBox::U32(v) => v,
                                _ => unreachable!()
                            };
                            match type_tag {
                                Tag::I64 => Ok(SupportedTypeBox::I64(lhs as i64)),
                                Tag::U64 => Ok(SupportedTypeBox::U64(lhs as u64)),
                                Tag::F64 => Ok(SupportedTypeBox::F64(lhs as f64)),
                                _ => {
                                    Err(anyhow!("{type_tag:?} cannot be casted to {into:?}"))
                                }
                            }
                        }
                        _ => Err(anyhow!("{type_tag:?} cannot be casted to {into:?}"))
                    }
                } else {
                    bail!("{tp:?} is not supported type")
                }
            }

            Cast::Propagated(a) => a.evaluate(interpreter)
        }
    }
}

impl CanBeEvaluated for Multiplicative {
    type Err = anyhow::Error;

    fn evaluate(&self, interpreter: &TestInterpreter) -> Result<SupportedTypeBox, Self::Err> {
        match self {
            Multiplicative::Binary { operator, lhs, rhs } => {
                let lhs = lhs.evaluate(interpreter)?;
                let rhs = rhs.evaluate(interpreter)?;
                if lhs.tag() == rhs.tag() {
                    let the_tag = lhs.tag();
                    todo!()
                } else {
                    // FIXME: non-standard
                    if operator == MultiplicativeOps::Multiply && lhs.tag() == Tag::String && rhs.tag() == Tag::I32 {
                        Ok(SupportedTypeBox::String(lhs.get_string().unwrap().repeat(rhs.get_i32().unwrap())))
                    } else {
                        bail!("{lhs:?} {rhs:?} {operator:?}")
                    }
                }
            }
            Multiplicative::Propagated(a) => a.evaluate(interpreter)
        }
    }
}

impl CanBeEvaluated for Additive {
    type Err = anyhow::Error;

    fn evaluate(&self, interpreter: &TestInterpreter) -> Result<SupportedTypeBox, Self::Err> {
        match self {
            Additive::Binary { operator, lhs, rhs } => {
                let lhs = lhs.evaluate(interpreter)?;
                let rhs = rhs.evaluate(interpreter)?;
                todo!()
            }
            Additive::Propagated(u) => u.evaluate(interpreter)
        }
    }
}
enum InterpreterError {
    SyntaxError,
    ExecutionError(anyhow::Error),
}

#[derive(Clone, PartialEq, Debug)]
enum SupportedTypeBox {
    Bool(bool),
    I8(i8),
    U8(u8),
    I16(i16),
    U16(u16),
    I32(i32),
    U32(u32),
    I64(i64),
    U64(u64),
    F32(f32),
    F64(f64),
    String(String),
}

impl SupportedTypeBox {
    fn tag(&self) -> SupportedTypeTag {
        match self {
            SupportedTypeBox::Bool(_) => SupportedTypeTag::Bool,
            SupportedTypeBox::I8(_) => SupportedTypeTag::I8,
            SupportedTypeBox::U8(_) => SupportedTypeTag::U8,
            SupportedTypeBox::I16(_) => SupportedTypeTag::I16,
            SupportedTypeBox::U16(_) => SupportedTypeTag::U16,
            SupportedTypeBox::I32(_) => SupportedTypeTag::I32,
            SupportedTypeBox::U32(_) => SupportedTypeTag::U32,
            SupportedTypeBox::I64(_) => SupportedTypeTag::I64,
            SupportedTypeBox::U64(_) => SupportedTypeTag::U64,
            SupportedTypeBox::F32(_) => SupportedTypeTag::F32,
            SupportedTypeBox::F64(_) => SupportedTypeTag::F64,
            SupportedTypeBox::String(_) => SupportedTypeTag::String,
        }
    }

    fn get_string(&self) -> Result<&String, anyhow::Error> {
        match self {
            SupportedTypeBox::String(str) => Ok(str),
            _ => bail!("{self:?} does not contain string")
        }
    }

    fn get_i32(&self) -> Result<i32, anyhow::Error> {
        match self {
            SupportedTypeBox::I32(v) => Ok(*v),
            _ => bail!("{self:?} does not contain i32")
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum SupportedTypeTag {
    Bool,
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
    F32,
    F64,
    String,
    Impulse,
}
