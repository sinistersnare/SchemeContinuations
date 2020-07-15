//! A shitty scheme interpreter that I made without
//! looking at too much inspiration.
//! I made this to study different continuation implementations.

//! TODO: QUOTED LISTS!!! will make life a lot easier! '(1 2 3)

use std::env;
use std::fs;

fn main() {
   let source_file = env::args().nth(1);
   println!("{:?}", source_file);
   let program = if let Some(filename) = env::args().nth(1) {
      fs::read_to_string(filename).expect("Could not read file")
   } else {
      // println!("REPL not available!");
      // process::exit(0)
      "(+ 2 2)".into()
   };

   println!("Lexing {:?}", program.trim());
   let tokens = match lex(&program) {
      Ok(t) => t,
      Err(e) => {println!("Error parsing: {:?}", e) ; return;},
   };
   println!("Lexed! {:?}", tokens);
   let (ast, _) = form_ast(&tokens, vec![]).expect("Couldnt form AST");
   // we wrap it in an extra vec unnecessarily. TODO: dont do that?
   let inner = match ast {
      Ast::Atom(t) => panic!("Should be wrapped in a vec cause im bad at coding"),
      Ast::List(mut vec) => vec.remove(0),
   };
   println!("AST: {:?}", inner);
}

/// these are literal strings found in the source, not types.
#[derive(Debug, Clone)]
enum Token {
   LeftParen,
   RightParen,
   Datum(String),
   Symbol(String),
   Numeric(f64), // TODO: support, strings, etc? Other types?
}

#[derive(Clone, Debug)]
enum LexState {
   Empty,
   MidDatum(String),
   MidNumeric(String),
   MidSymbol(String),
}

/// wow. such simple. very small. lisp.
#[derive(Debug)]
enum Ast {
   Atom(Token),
   List(Vec<Ast>),
}

#[derive(Debug)]
enum Expr {
   Datum(String),
   Symbol(String),
   Numeric(f64),
   If(Box<Expr>, Box<Expr>, Box<Expr>),
   Define(String, Vec<Expr>, Box<Expr>),
   Application(Box<Expr>, Vec<Expr>),
}

// TODO: theres gotta be a better way?? this code fucking sucks
fn lex(code: &str) -> Result<Vec<Token>, String> {
   fn clear_state(tokens: &mut Vec<Token>, state: &mut LexState) -> Result<(), String> {
      match state {
         LexState::Empty => {},
         LexState::MidDatum(dat) => {
            tokens.push(Token::Datum(dat.clone()));
         },
         LexState::MidNumeric(num) => {
            let parsed: f64 = match num.parse() {
               Ok(n) => n,
               Err(_) => {
                  return Err(format!("Cant Parse numeric! {}", num));
               },
            };
            tokens.push(Token::Numeric(parsed));
         },
         LexState::MidSymbol(symbol) => {
            tokens.push(Token::Symbol(symbol.clone()));
         },
      }
      *state = LexState::Empty;
      Ok(())
   }

   let mut tokens = Vec::with_capacity(1024);
   let mut state = LexState::Empty;
   // TODO: use `i` to have code spans.
   for (_i, c) in code.chars().enumerate() {
      match c {
         '(' => {
            clear_state(&mut tokens, &mut state)?;
            tokens.push(Token::LeftParen);
         },
         ')' => {
            clear_state(&mut tokens, &mut state)?;
            tokens.push(Token::RightParen);
         },
         '\'' => {
            match &mut state {
               LexState::Empty => {
                  state = LexState::MidSymbol(String::with_capacity(16));
               },
               LexState::MidDatum(dat) => {
                  return Err(format!("Found a ' mid datum! datum state was: {:?}, tokens so far: {:?}", 
                                    dat, tokens));
               },
               LexState::MidNumeric(num) => {
                  return Err(format!("Found a ' mid numeric! numeric state was: {:?}, tokens so far: {:?}", 
                                    num, tokens));

               },
               LexState::MidSymbol(ref mut symbol) => {
                  symbol.push('\'');
               },
            }
         },
         '.' => {
            match &mut state {
               LexState::Empty => {
                  let mut num_string = String::with_capacity(16);
                  num_string.push('.');
                  state = LexState::MidNumeric(num_string);
               },
               LexState::MidDatum(dat) => {
                  return Err(format!("Found a . mid datum! datum state was: {:?}, tokens so far: {:?}", 
                                    dat, tokens));
               },
               LexState::MidNumeric(ref mut num) => {
                  if num.contains('.') {
                     return Err(format!("the numeric already has a dot! numeric state was: {:?}, tokens so far: {:?}", 
                                          num, tokens));
                  } else {
                     num.push('.');
                  }
               },
               LexState::MidSymbol(ref mut symbol) => {
                  symbol.push('.');
               },
            }
         }
         c @ '0'..='9' => {
            match &mut state {
               LexState::Empty => {
                  let mut numstring = String::with_capacity(16);
                  numstring.push(c);
                  state = LexState::MidNumeric(numstring);
               },
               LexState::MidDatum(ref mut dat) => {
                  dat.push(c);
               },
               LexState::MidNumeric(ref mut num) => {
                  num.push(c);

               },
               LexState::MidSymbol(ref mut symbol) => {
                  symbol.push(c);
               },
            }
         },
         c @ '*'..='-' | c @ '/' | c @ '<'..='Z' | c @ 'a'..='z' => {
            match &mut state {
               LexState::Empty => {
                  let mut datstring = String::with_capacity(16);
                  datstring.push(c);
                  state = LexState::MidDatum(datstring);
               },
               LexState::MidDatum(ref mut dat) => {
                  dat.push(c);
               },
               LexState::MidNumeric(num) => {
                  return Err(format!("Cant put an alpha char into a numeric! numeric state was: {:?}, tokens so far: {:?}", 
                                       num, tokens));
               },
               LexState::MidSymbol(ref mut symbol) => {
                  symbol.push(c);
               },
            }
         },
         ' ' | '\t' | '\n' => {
            clear_state(&mut tokens, &mut state)?;
         },
         other => {
            return Err(format!("Character '{:?}' not supported! sorry!", other));
         }
      }
   }
   clear_state(&mut tokens, &mut state)?;
   Ok(tokens)
}

fn form_ast(tokens: &[Token], mut formed: Vec<Ast>) -> Result<(Ast, usize), String> {
   let mut i = 0;
   loop {
      if i >= tokens.len() {
         return Ok((Ast::List(formed), i));
      }
      match &tokens[i] {
         Token::LeftParen => {
            let (inner_ast, advanced) = form_ast(&tokens[i+1..], vec![])?;
            formed.push(inner_ast);
            // + 1 to include the left paren.
            i += advanced + 1;
         },
         Token::RightParen => {
            // there are no 'inner' parens, because this is a 
            // recursive function, so each set of parens would be
            // the 'only' one...
            // FIXME: how to deal with bad syntax like "())"
            return Ok((Ast::List(formed), i + 1));
         },
         Token::Datum(dat) => {
            formed.push(Ast::Atom(Token::Datum(dat.clone())));
            i += 1;
         },
         Token::Symbol(sym) => {
            formed.push(Ast::Atom(Token::Symbol(sym.clone())));
            i += 1;
         },
         Token::Numeric(num) => {
            formed.push(Ast::Atom(Token::Numeric(*num)));
            i += 1;
         },
      }
   }
}

fn parse(tokens: &[Token]) -> Result<Expr, String> {

   // match &tokens[0] {
   //       Token::LeftParen => {
   //          // TODO: better type for this than u32?
   //          let mut paren_balance = 0u32;
   //          for i in 1..tokens.len() {
   //             match &tokens[i] {
   //                Token::LeftParen => {
   //                   paren_balance += 1;
   //                },
   //                Token::RightParen => {
   //                   if paren_balance == 0 {
   //                      return parse
   //                   }
   //                }
   //             }
   //          }
   //       },
   //       Token::RightParen => {
   //          return Err("Unbalanced right paren!".into());
   //       },
   //       Token::Datum(dat) => {
   //       },
   //       Token::Symbol(sym) => {},
   //       Token::Numeric(num) => {},
   // }

   // when this is 0 there are no unclosed parens
   // when its positive there are parens that need to be closed
   // when its negative, we are in a bad state, fail parsing.
   let mut paren_balance = 0;
   for i in 0..tokens.len() {
      let cur = &tokens[i];
      match cur {
         Token::LeftParen => {
            paren_balance += 1;
         },
         Token::RightParen => {
            paren_balance -= 1;
         },
         Token::Datum(dat) => {},
         Token::Symbol(sym) => {},
         Token::Numeric(num) => {},
      }
   }
   Ok(Expr::Numeric(12.34))
}