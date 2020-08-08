//! A shitty scheme interpreter that I made without
//! looking at too much inspiration
//! JK I ENDED UP USING https://github.com/rui314/minilisp/blob/master/minilisp.c
//! FOR A LOT OF INSPIRATION!!!! TY MINILISP!
//!
//! I made this to study different continuation implementations.

use std::env;
use std::fs;

use generational_arena as arena;

pub mod read;
pub mod eval;
pub mod prims;

use read::{ReadResult, Parser};
use prims::PrimFunc;
use eval::Evaluator;

fn main() {
   if let Some(filename) = env::args().nth(1) {
      exec_string(fs::read_to_string(filename).expect("Could not read file"))
   } else {
      start_repl();
   };
}

fn start_repl() {
   use std::io::{stdin, stdout, Write};
   let mut input = String::new();
   let mut evaluator = Evaluator::new();

   loop {
      print!("> ");
      stdout().flush().expect("Flushed poorly.");
      stdin().read_line(&mut input).expect("Did not enter a full string.");
      let mut parser = Parser::new(input.trim().to_string());
      loop {
         let expr = parser.read_expr(&mut evaluator);
         match expr {
            ReadResult::Expression(parsed) => {
               evaluator.eval(parsed);
               input.clear();
            },
            ReadResult::EOF => {
               break;
            },
            ReadResult::Dot => {
               panic!("Unexpected dot `.`!");
            },
            ReadResult::CloseParen => {
               panic!("Unbalanced close paren!");
            },
            ReadResult::Error(e) => {
               panic!("Got an error while parsing an exp: {:?}", e);
            },
         }}
   }
}

fn exec_string(program: String) {
   println!("Parsing {:?}", program.trim());

   let mut parser = Parser::new(program.trim().to_string());
   let mut evaluator = Evaluator::new();

   loop {
      let expr = parser.read_expr(&mut evaluator);
      match expr {
         ReadResult::Expression(parsed) => {
            evaluator.eval(parsed);
         },
         ReadResult::CloseParen => {
            panic!("Unbalanced close paren!");
         },
         ReadResult::Dot => {
            panic!("Unexpected dot `.`!");
         },
         ReadResult::Error(e) => {
            panic!("Got an error while parsing an exp: {:?}", e);
         },
         ReadResult::EOF => {
            return;
         },
      }
   }
}

pub enum ScmObj {
   /// A number. All numbers in this language
   /// are double precision floating points.
   /// Because youre not special enough to need integers.
   Numeric(f64),
   /// A Symbol, which is any text.
   /// examples: `+`, `hello`, `&lol`
   Symbol(String),
   /// A boolean value, true or false.
   Bool(bool),
   /// A Cons cell, which has a first and second object.
   /// this is often used to create a linked list.
   /// TODO: is it OK that these are in boxes?
   ///      I think these need to be allocated in the
   ///      languages heap, not the Rust heap.
   Cons(arena::Index, arena::Index),
   /// used to signal an empty list. represented by '()
   /// which is (quote ()).
   Null,
   /// the complete absence of a value. usually returned by functions
   /// like print
   Void,
   /// A primitive function, implemented by the interpreter.
   Primitive(PrimFunc),
   /// A scheme function. Contains a list of formal params, and a body (which is a ScmObj).
   Func(Vec<String>, arena::Index),
   /// Probably shouldnt be a thing :p.
   Other,
   // unimplemented types.
   // Closure, Int, Str, Vector, Hash, Set
}

impl std::fmt::Debug for ScmObj {
   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      match *self {
         ScmObj::Numeric(n) => write!(f, "Numeric({})", n),
         ScmObj::Symbol(ref s) => write!(f, "Symbol({})", s),
         ScmObj::Null => write!(f, "'()"),
         ScmObj::Bool(true) => write!(f, "#t"),
         ScmObj::Bool(false) => write!(f, "#f"),
         ScmObj::Void => write!(f, "#<void>"),
         ScmObj::Cons(ref car, ref cdr) => write!(f, "#<cons>"),
         ScmObj::Func(..) => write!(f, "#<function>"),
         ScmObj::Other => write!(f, "Other Thing! This shouldnt exist!"),
         // TODO: include prim name?
         ScmObj::Primitive(_p) => write!(f, "#<primitive>"),
      }
   }
}

/// in this language, the only falsy value is #f (false).
/// Everything else is true!
/// is_truthy_value(&mut ScmObj::Null) ==> true
pub fn is_truthy_value(val: &ScmObj) -> bool {
   if let ScmObj::Bool(false) = val { false } else { true }
}
