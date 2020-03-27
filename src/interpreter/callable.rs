use crate::interpreter::Interpreter;
use std::fmt::Debug;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::parser::ast::{Literal, Stmt};
use crate::lexer;
use std::fmt;
use crate::environment::{Object, Environment};
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;

pub trait Callable: Debug {
    type Result;

    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Self::Result>) -> Option<Self::Result>;
    fn arity(&self) -> usize;
    fn to_string(&self) -> String;
}

#[derive(Debug)]
pub struct Clock;

impl Callable for Clock {
    type Result = Object;

    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Self::Result>) -> Option<Self::Result> {
        let start = SystemTime::now();
        Some(Object::L(Literal::Float(start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs_f64())))
    }

    fn arity(&self) -> usize {
        0
    }

    fn to_string(&self) -> String {
        "<native fn>".to_owned()
    }
}

impl fmt::Display for Clock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<native fn clock>")
    }
}

#[derive(Debug)]
pub struct Function {
    pub declaration: Stmt
}

impl Callable for Function {
    type Result = Object;

    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Self::Result>) -> Option<Self::Result> {
        if let Stmt::Function { ref parameters, ref body, .. } = self.declaration {
            let mut global_values = HashMap::new();
            for (name, object) in interpreter.globals.borrow().values.borrow().iter() {
                global_values.insert(name.to_owned(), object.clone());
            }
            let mut env = Environment { enclosing: None, values: Rc::new(RefCell::new(global_values)) };
            if let Some(parameters) = parameters {
                for (i, param) in parameters.iter().enumerate() {
                    if let lexer::Token::Identifier(param) = param {
                        let arg = arguments.get(i).unwrap();
                        env.define(param.as_str(), arg.clone());
                    }
                };
            }

            if let Stmt::Block(body) = body.clone().deref() {
                interpreter.execute_block(body, env);
                return None;
            } else { panic!() }
        } else { panic!() }
    }

    fn arity(&self) -> usize {
        if let Stmt::Function { ref parameters, ..} = self.declaration {
            parameters.clone().map_or(0, |p| p.len())
        } else { panic!() }
    }

    fn to_string(&self) -> String {
        if let Stmt::Function { ref name, .. } = self.declaration {
            if let lexer::Token::Identifier(name) = name {
                return name.to_string();
            }
        }
        panic!()
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Stmt::Function { ref name, .. } = self.declaration {
            write!(f, "{}", name)
        } else {
            panic!()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::lexer;
    use crate::parser::parser::Parser;
    use crate::interpreter::Interpreter;

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
        Interpreter::new().interpret(e.as_ref());
    }
}