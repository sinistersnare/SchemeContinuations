//! A shitty scheme interpreter that I made without
//! looking at too much inspiration
//! JK I ENDED UP USING https://github.com/rui314/minilisp/blob/master/minilisp.c
//! FOR A LOT OF INSPIRATION!!!! TY MINILISP!
//! I made this to study different continuation implementations.

use std::env;
use std::fs;

pub mod read;
pub mod eval;
pub mod prims;
pub use read::{ReadResult, Parser};
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
      let _ = stdout().flush();
      stdin().read_line(&mut input).expect("Did not enter a full string.");
      let mut parser = Parser::new(input.trim().to_string());
      loop {
         let expr = parser.read_expr();
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
            panic!("Got an error while parsing an exp: {:?}", e);
         },
         ReadResult::EOF => {
            return;
         },
      }
   }
}
