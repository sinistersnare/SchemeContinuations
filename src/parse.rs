
//! A Parser of Scheme types.
//! `Expr` is the AST type here that will be executed.
//!
//! TODO: rename this to 'read.rs'? seems right, with eval.rs.

/*
   /// an if conditional, with a then and else section
   If(Box<Expr>, Box<Expr>, Box<Expr>),
   /// a function definition, a name, args (list of datum), and a body.
   Define(String, Vec<Expr>, Box<Expr>),
*/

/// TODO: string type?
#[derive(Debug)]
pub enum Expr {
   /// a name of something, like `foo`
   Symbol(String),
   /// a number, only f64 is supported in this lisp.
   Numeric(f64),
   /// can be function application `(f a b c)` or perhaps a list
   /// that is being quoted like (quote (foo bar baaz))
   /// which is a special form, but saying this for example
   /// purposes.
   /// This should represent a null-terminated list, like
   /// (cons a (cons b (cons c '())))
   List(Vec<Expr>),
   /// hmmm couldnt think of a better way to do this,
   /// this way a 'Dot' expr isnt needed... right???
   /// an improper list is a list that DOESNT end in null.
   /// (cons a (cons b c)) <==> '(a b . c)
   ImproperList(Vec<Expr>),
   Null,
}

/// TODO: not a 'rust result', needa rename.
#[derive(Debug)]
pub enum ParseResult {
   Expression(Expr),
   CloseParen,
   Dot,
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

   fn read_symbol(&mut self, mut symstr: String) -> Expr {
      // TODO we should intern these symbols? I think so.
      //    the interner should be a member of Self
      loop {
         if let Some(peeked) = self.peek() {
            // TODO: check allowed symbols in another TODO elsewhere.
            if peeked.is_alphabetic() || peeked.is_digit(10) {
               self.take(); // take it after we know we want it.
               symstr.push(peeked);
            } else {
               return Expr::Symbol(symstr);
            }
         } else {
            return Expr::Symbol(symstr);
         }
      }
   }

   /// already read the 'expr and returns (quote expr)
   fn read_quote(&mut self) -> Expr {
      // read in the next expr and wrap it in `(quote <expr>)`.
      let next_expr = self.read_expr();
      match next_expr {
         ParseResult::Expression(e) => {
            let list = vec![Expr::Symbol("quote".to_string()), e];
            Expr::List(list)
         },
         ParseResult::Dot => panic!("Illegal use of `.`"),
         ParseResult::CloseParen => panic!("Illegal use of `)`"),
         ParseResult::EOF => {
            panic!("Tried reading a quoted thing but got EOF!");
         }
      }
   }

   // reads a number, its been started already somehow,
   // like if it saw a `-` followed by a number,
   // or a `.` followed by a number.
   // or a number followed by a number.
   fn read_number(&mut self, mut numstr: String) -> Expr {
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
      Expr::Numeric(numstr.parse().expect("Wasnt able to parse as a f64."))
   }

	/// already got the open paren before this was called.
	/// we now look for list elements (expressions),
	/// a dot, followed by a final element, then a CloseParen.
	///    (this forms an improper list).
	/// or a close paren, ending the list.
   fn read_list(&mut self) -> Expr {
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
         return Expr::Null;
      }

      let mut list = Vec::with_capacity(16);
      loop {
         let res = self.read_expr();
         match res {
            ParseResult::Expression(e) => {
               list.push(e);
            },
            // finishes an improper list.
            ParseResult::Dot => {
               // take the last element,
               // then take a close paren,
               // but dont end the list with a Null.
               let last = self.read_expr();
               match last {
                  ParseResult::Expression(final_e) => {
                     let closer = self.take();
                     if let Some(')') = closer {
                     	list.push(final_e);
                     	return Expr::ImproperList(list);
                     } else {
                        panic!("Expected ')' found {:?}", closer);
                     }
                  },
                  ParseResult::Dot => panic!("Expected an expression, got `.`"),
                  ParseResult::CloseParen => panic!("Expected an expression, got `)`"),
                  ParseResult::EOF => panic!("expected an expression, got EOF!"),
               }
            },
            ParseResult::CloseParen => {
               // properly end the list.
               return Expr::List(list);
            },
            ParseResult::EOF => {
               panic!("Got EOF mid list parse!");
            }
         }
      }
   }

   /// TODO: i dont think this is a great function to make public.
   /// Maybe it should be repackaged as an iterator of Result<Expression>
   pub fn read_expr(&mut self) -> ParseResult {
      loop {
         let took = self.take();
         if let None = took {
            return ParseResult::EOF;
         }
         let c = took.unwrap();
         match c {
            // whitespace insensitive syntax!
            ' ' | '\n' | '\t' | '\r' => {},
            // comment
            ';' => { self.skip_line(); },
            '(' => {
               return ParseResult::Expression(self.read_list());
            },
            ')' => {
               return ParseResult::CloseParen;
            },
            // a number can be started simply
            '0'..='9' => {
               let mut numstr = String::with_capacity(16);
               numstr.push(c);
               return ParseResult::Expression(self.read_number(numstr));
            },
            // a number can be started with a `-` to signify a negative number.
            // or it can be referencing a function called `-`.
            '-' => {
               let peeked_opt = self.peek();
               if let Some(peeked) = peeked_opt {
                  if peeked.is_digit(10) || peeked == '.' {
                     let mut numstr = String::with_capacity(16);
                     numstr.push(c);
                     return ParseResult::Expression(self.read_number(numstr));
                  } else {
                     let mut symstr = String::with_capacity(16);
                     symstr.push(c);
                     return ParseResult::Expression(self.read_symbol(symstr));
                  }
               } else {
                  // fast path!
                  return ParseResult::Expression(Expr::Symbol("-".to_string()));
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
                     return ParseResult::Expression(self.read_number(numstr));
                  } else {
                     return ParseResult::Dot;
                  }
               } else {
                  panic!("Unexpected . before EOF!");
               }
            },
            '\'' => {
               return ParseResult::Expression(self.read_quote());
            },
            // TODO: this is ugly AF lol.
            c@'<'..='Z' | c@'a'..='z' | c@'~'
               | c@'!' | c@'#' | c@'$' | c@'%'
               | c@'^' | c@'&' | c@'*' | c@'_'
               | c@'+' | c@':' | c@'/' => {
               let mut symstr = String::with_capacity(16);
               symstr.push(c);
               return ParseResult::Expression(self.read_symbol(symstr));
            },
            _ => {
               // TODO: maybe a ParseResult::Error would be cool?
               return ParseResult::Expression(Expr::Symbol("TODO_ELSE!".to_string()));
            },
         }
      }
   }
}
