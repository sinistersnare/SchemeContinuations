//! A Parser of Scheme types.
//! `Expr` is the AST type here that will be executed.

use crate::eval::ScmObj;

/// TODO: not a 'rust result', needa rename.
#[derive(Debug)]
pub enum ReadResult {
   Expression(ScmObj),
   CloseParen,
   Dot,
   Error(&'static str),
   EOF,
}

pub struct Parser {
   text: String,
}

impl Parser {
   pub fn new(text: String) -> Parser {
      Parser {
         text,
      }
   }

   fn take(&mut self) -> Option<char> {
      if self.text.is_empty() {
         None
      } else {
         // TODO: removing probably not a good idea
         // does it move everything, or just bump the ptr?
         // check and just advance a pos if so.
         Some(self.text.remove(0))
      }
   }

   fn peek(&mut self) -> Option<char> {
      // theres def a better way to get a char from this.
      self.text.chars().peekable().peek().map(|&c| c)
   }

   fn skip_line(&mut self) {
      loop {
         // TODO: i think Option::contains would work here
         //       once its stabilized.
         if let Some(c) = self.take() {
            if c == '\n' {
               return;
            }
         } else {
            return;
         }
      }
   }

   fn read_symbol(&mut self, mut symstr: String) -> ScmObj {
      // TODO we should intern these symbols? I think so.
      //    the interner should be a member of Self
      loop {
         if let Some(peeked) = self.peek() {
            // TODO: check allowed symbols in another TODO elsewhere.
            if peeked.is_alphabetic() || peeked.is_digit(10) {
               self.take(); // take it after we know we want it.
               symstr.push(peeked);
            } else {
               return ScmObj::Symbol(symstr);
            }
         } else {
            return ScmObj::Symbol(symstr);
         }
      }
   }

   /// already read the 'expr and returns (quote expr)
   fn read_quote(&mut self) -> ScmObj {
      // read in the next expr and wrap it in `(quote <expr>)`.
      let next_expr = self.read_expr();
      match next_expr {
         ReadResult::Expression(e) => {
            ScmObj::Cons(Box::new(ScmObj::Symbol("quote".into())),
                         Box::new(e))
         },
         ReadResult::Dot => panic!("Illegal use of `.`"),
         ReadResult::CloseParen => panic!("Illegal use of `)`"),
         ReadResult::EOF => {
            panic!("Tried reading a quoted thing but got EOF!");
         }
         ReadResult::Error(e) => panic!(e),
      }
   }

   // reads a number, its been started already somehow,
   // like if it saw a `-` followed by a number,
   // or a `.` followed by a number.
   // or a number followed by a number.
   fn read_number(&mut self, mut numstr: String) -> ScmObj {
      while let Some(c) = self.peek() {
         if c.is_digit(10) || c == '.' || "~!@#$%^&*-_=+:/?<>".contains(c) {
            // take it when we know its a digit we want.
            self.take();
            numstr.push(c);
         } else {
            // we have ended appropriate digit characters
            // so leave the loop.
            break;
         }
      }
      // TODO: proper error handing.
      ScmObj::Numeric(numstr.parse().expect("Wasnt able to parse as a f64."))
   }

	/// already got the open paren before this was called.
	/// we now look for list elements (expressions),
	/// a dot, followed by a final element, then a CloseParen.
	///    (this forms an improper list).
	/// or a close paren, ending the list.
   fn read_list(&mut self) -> ScmObj {
      if let Some(')') = self.peek() {
      	// I _THINK_ this is a hack. IDK THO LOL.
      	// like, idk if Null should be something we actually look for.
      	// hopefully not, and i can delete this???
         // if what we are reading is just a `()`,
         // then just return Null.
         // if the expression is `()` and not `'()`
         // im pretty sure its illegal, but... couldnt think of a
         // good way to just do '() and not ().
         self.take();
         return ScmObj::Null;
      }

      let cur = self.read_expr();
      match cur {
         ReadResult::CloseParen => {
            ScmObj::Null
         },
         ReadResult::Expression(e) => {
            ScmObj::Cons(Box::new(e), Box::new(self.read_list()))
         },
         ReadResult::Dot => {
            let improper_final = self.read_expr();
            let close_paren = self.take();
            if let Some(')') = close_paren {
               match improper_final {
                  ReadResult::Expression(e) => e,
                  ReadResult::Dot => panic!("Expected an expression after a `.`, got `.`"),
                  ReadResult::CloseParen => panic!("Expected an expression after a `.`, got `)`"),
                  ReadResult::EOF => panic!("expected an expression after a `.`, got EOF!"),
                  ReadResult::Error(e) => panic!(e),
               }
            } else {
               panic!("Expected ')', found {:?}.", close_paren);
            }
         },
         ReadResult::EOF => { panic!("Got EOF mid list parse!"); },
         ReadResult::Error(e) => { panic!("Error while reading an expr: {:?}", e); },
      }
   }

   /// TODO: i dont think this is a great function to make public.
   /// Maybe the public API  should be an iterator of
   /// Result<Expression> and use that one publicly.
   pub fn read_expr(&mut self) -> ReadResult {
      loop {
         let took = self.take();
         if let None = took {
            return ReadResult::EOF;
         }
         let c = took.unwrap();
         match c {
            // whitespace insensitive syntax!
            ' ' | '\n' | '\t' | '\r' => {},
            // comment
            ';' => { self.skip_line(); },
            '(' => {
               return ReadResult::Expression(self.read_list());
            },
            ')' => {
               return ReadResult::CloseParen;
            },
            // a number can be started simply
            '0'..='9' => {
               let mut numstr = String::with_capacity(16);
               numstr.push(c);
               return ReadResult::Expression(self.read_number(numstr));
            },
            // a number can be started with a `-` to signify a negative number.
            // or it can be referencing a function called `-`.
            '-' => {
               let peeked_opt = self.peek();
               if let Some(peeked) = peeked_opt {
                  if peeked.is_digit(10) || peeked == '.' {
                     let mut numstr = String::with_capacity(16);
                     numstr.push(c);
                     return ReadResult::Expression(self.read_number(numstr));
                  } else {
                     let mut symstr = String::with_capacity(16);
                     symstr.push(c);
                     return ReadResult::Expression(self.read_symbol(symstr));
                  }
               } else {
                  // fast path!
                  return ReadResult::Expression(ScmObj::Symbol("-".to_string()));
               }
            },
            // can also start a number just with a `.` i.e. `.5` == `0.5`.
            // this can also be a dot used for lisp cons stuff (a . b)
            '.' => {
               let peeked_opt = self.peek();
               if let Some(peeked) = peeked_opt {
                  if peeked.is_digit(10) {
                     let mut numstr = String::with_capacity(16);
                     numstr.push(c);
                     return ReadResult::Expression(self.read_number(numstr));
                  } else {
                     return ReadResult::Dot;
                  }
               } else {
                  panic!("Unexpected . before EOF!");
               }
            },
            '\'' => {
               return ReadResult::Expression(self.read_quote());
            },
            // TODO: this is ugly AF lol.
            c@'<'..='Z' | c@'a'..='z' | c@'~'
               | c@'!' | c@'#' | c@'$' | c@'%'
               | c@'^' | c@'&' | c@'*' | c@'_'
               | c@'+' | c@':' | c@'/' => {
               let mut symstr = String::with_capacity(16);
               symstr.push(c);
               return ReadResult::Expression(self.read_symbol(symstr));
            },
            _ => {
               // TODO: maybe a ReadResult::Error would be cool?
               return ReadResult::Expression(ScmObj::Symbol("TODO_ELSE".into()));
               // return ReadResult::Expression(Expr::Symbol("TODO_ELSE!".to_string()));
            },
         }
      }
   }
}
