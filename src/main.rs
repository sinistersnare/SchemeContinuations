//! A shitty scheme interpreter that I made without
//! looking at too much inspiration
//! JK I ENDED UP USING https://github.com/rui314/minilisp/blob/master/minilisp.c
//! FOR A LOT OF INSPIRATION!!!! TY MINILISP!
//!
//! I made this to study different continuation implementations.
#[macro_use]
extern crate lazy_static;
extern crate combine;
extern crate im;

use std::env;
use std::fs;

use combine::Parser;

pub mod prims;
pub mod common;
pub mod eval;
pub mod read;

use crate::eval::evaluate;
use crate::read::expr;

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

   loop {
      print!("> ");
      stdout().flush().expect("Flushed poorly...?");
      stdin()
         .read_line(&mut input)
         .expect("Did not enter a full string.");
      let parsed = expr().parse(input.trim());
      match parsed {
         Ok((sexpr, _)) => {
            let (final_val, _fin_state, _store) = evaluate(sexpr);
            println!("{:?}", final_val);
         }
         Err(e) => println!("Error Parsing: {:?}", e),
      };
      input.clear();
   }
}

fn exec_string(program: String) {
   let parsed = expr().parse(program.trim());
   match parsed {
      Ok((sexpr, _)) => {
         let (final_val, _fin_state, _store) = evaluate(sexpr);
         println!("{:?}", final_val);
      }
      Err(e) => println!("Error Parsing: {:?}", e),
   };
}
