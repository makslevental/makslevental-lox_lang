use crate::interpreter::Interpreter;
use crate::lexer;
use crate::parser::ast::{Literal, Stmt};
use crate::symbol_table::{Object, SymbolTable};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Debug, Display};
use std::ops::Deref;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

pub trait Callable: Debug + Display {
    type Result;

    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Self::Result>,
    ) -> Option<Self::Result>;
    fn arity(&self) -> usize;
}

#[derive(Debug)]
pub struct Clock;

impl Callable for Clock {
    type Result = Object;

    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Self::Result>,
    ) -> Option<Self::Result> {
        let start = SystemTime::now();
        Some(Object::L(Literal::Float(
            start
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs_f64(),
        )))
    }

    fn arity(&self) -> usize {
        0
    }
}

impl fmt::Display for Clock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<native fn clock>")
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    pub declaration: Stmt,
    pub closure: SymbolTable,
}

impl Callable for Function {
    type Result = Object;

    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Self::Result>,
    ) -> Option<Self::Result> {
        if let Stmt::Function {
            ref parameters,
            ref body,
            ..
        } = self.declaration
        {
            let mut rec_existing = SymbolTable {
                enclosing: None,
                values: Rc::new(RefCell::new(Default::default())),
            };
            let mut env = self.closure.deep_copy();
            if let Some(parameters) = parameters {
                for (i, param) in parameters.iter().enumerate() {
                    if let lexer::Token::Identifier(param) = param {
                        let arg = arguments.get(i).unwrap();
                        if env.exists(param) {
                            rec_existing.define(param.as_str(), arg.clone());
                        }
                        env.define(param.as_str(), arg.clone());
                    }
                }
            }

            if let Stmt::Block(body) = body.clone().deref() {
                let ret_env = interpreter.execute_block(body, env);
                let names: Vec<String> = self.closure.values.borrow().keys().map(|k| String::from(k)).collect();
                for key in names {
                    self.closure
                        .values
                        .borrow_mut()
                        .insert(key.to_string(), ret_env.get(&key));
                }
                interpreter.ret.take().map_or(None, |r| r.right())
            } else {
                panic!()
            }
        } else {
            panic!()
        }
    }

    fn arity(&self) -> usize {
        if let Stmt::Function { ref parameters, .. } = self.declaration {
            parameters.clone().map_or(0, |p| p.len())
        } else {
            panic!()
        }
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Stmt::Function { ref name, .. } = self.declaration {
            write!(f, "<fn {}>", name)
        } else {
            panic!()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::interpreter::Interpreter;
    use crate::lexer::lexer;
    use crate::parser::parser::Parser;

    #[test]
    fn clock() {
        let input: Vec<char> = r#"
            print clock();
        "#
        .chars()
        .collect();
        let tokens = lexer().parse(&input).unwrap();
        let mut p = Parser::new(tokens);
        let e = p.parse();
        Interpreter::new().interpret(e.as_ref());
    }

    #[test]
    fn count() {
        let input: Vec<char> = r#"
            fun count(n) {
              if (n > 1) count(n - 1);
              print n;
            }

            count(3);
        "#
        .chars()
        .collect();
        let tokens = lexer().parse(&input).unwrap();
        let mut p = Parser::new(tokens);
        let e = p.parse();
        let mut i = Interpreter::new();
        i.interpret(e.as_ref());
    }

    #[test]
    fn closure() {
        let input: Vec<char> = r#"
            fun makeCounter() {
              var i = 0;
              fun count() {
                i = i + 1;
                print i;
              }

              return count;
            }

            var counter = makeCounter();
            counter();
            counter();
        "#
        .chars()
        .collect();
        let tokens = lexer().parse(&input).unwrap();
        let mut p = Parser::new(tokens);
        let e = p.parse();
        let mut i = Interpreter::new();
        i.interpret(e.as_ref());
    }

    #[test]
    fn function_global_mut() {
        let input: Vec<char> = r#"
            var d = 4;
            fun bob() {
                print d;
                d = 5;
                print d;
            }

            bob();
            print d;
        "#
        .chars()
        .collect();
        let tokens = lexer().parse(&input).unwrap();
        let mut p = Parser::new(tokens);
        let e = p.parse();
        Interpreter::new().interpret(e.as_ref());
    }
}
