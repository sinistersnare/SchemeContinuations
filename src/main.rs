//! A shitty scheme interpreter that I made without
//! looking at too much inspiration
//! JK I ENDED UP USING https://github.com/rui314/minilisp/blob/master/minilisp.c
//! FOR A LOT OF INSPIRATION!!!! TY MINILISP!
//! I made this to study different continuation implementations.

use std::env;
use std::fs;

pub mod read;
pub mod eval;
pub use read::{ReadResult, Parser};
use eval::Evaluator;

fn main() {
   let program = if let Some(filename) = env::args().nth(1) {
      fs::read_to_string(filename).expect("Could not read file")
   } else {
      // println!("REPL not available!");
      // "'(a b . c) (+ 1)".into()
      "(if 'boop (if 1 '(+ 12 12) 6) 3)".into()
   };

   println!("Parsing {:?}", program.trim());

   let mut parser = Parser::new(program.trim().to_string());
   let mut evaluator = Evaluator::new();

   // if let ParseResult::Expression(p) = parser.read_expr() {
   //    evaluator.eval(p);
   // }

   loop {
      let expr = parser.read_expr();
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
            panic!("Got an error while parsing an exp: {:?}", e)
         },
         ReadResult::EOF => {
            return;
         },
      }
   }
}
