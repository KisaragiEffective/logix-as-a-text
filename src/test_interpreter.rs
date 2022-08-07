use std::any::Any;
use std::collections::HashMap;
use crate::compiler::lexer::Lexer;
use crate::compiler::parser::{Identifier, Parser, RightHandSideValue, RootAst, Statement};

type ExecutionResult = Result<Vec<Box<dyn Any>>, InterpreterError>;

struct TestInterpreter {
    scope: HashMap<Identifier, Box<dyn Any>>
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
            match statement {
                Statement::NodeDeclaration { identifier, type_tag, rhs } => {
                    let type_tag = type_tag.expect("Currently, the node declaration must have explicit type annotation.\
                    Note: This is an implementation restriction, and will be removed in future. Please see https://github.com/KisaragiEffective/logix-as-a-text/issues/6\
                          for current status.");
                    match rhs {
                        RightHandSideValue::Identifier(ident) => {}
                        RightHandSideValue::MemberPath(path) => {}
                        RightHandSideValue::Expression(expr) => {

                        }
                    }
                }
                Statement::Comment { .. } => {
                    // NOP
                }
                Statement::NoMoreStatements => {}
            }
        }
    }
}

enum InterpreterError {
    SyntaxError,
    ExecutionError(anyhow::Error),
}