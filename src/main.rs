//! A shitty scheme interpreter that I made without
//! looking at too much inspiration
//! JK I ENDED UP USING https://github.com/rui314/minilisp/blob/master/minilisp.c
//! FOR A LOT OF INSPIRATION!!!! TY MINILISP!
//! I made this to study different continuation implementations.

use std::env;
use std::fs;

pub mod parse;
pub mod eval;
pub use parse::{ParseResult, Parser};
use eval::{Evaluator};

fn main() {
   let program = if let Some(filename) = env::args().nth(1) {
      fs::read_to_string(filename).expect("Could not read file")
   } else {
      // println!("REPL not available!");
      // "'(a b . c) (+ 1)".into()
      "(cons 'a (cons 'b ('c '())))".into()
   };

   println!("Parsing {:?}", program.trim());

   let mut parser = Parser::new(program.trim().to_string());
   let mut evaluator = Evaluator::new();

   loop {
      let expr = parser.read_expr();
      match expr {
         ParseResult::Expression(parsed) => {
            evaluator.eval(parsed);
            // now to execute each parsed expr!!!!!!!.
         },
         ParseResult::CloseParen => {
            panic!("Unbalanced close paren!");
         },
         ParseResult::Dot => {
            panic!("Unexpected dot `.`!");
         },
         ParseResult::EOF => {
            return;
         }
      }
   }
}
