//! The evaluator of scheme ASTs.

use std::collections::HashMap;

use im;
use generational_arena as arena;

use crate::prims::{self, PrimFunc};

// TODO: string type?
pub enum ScmObj {
   Numeric(f64),
   Symbol(String),
   Cons(Box<ScmObj>, Box<ScmObj>), // what type here hmmm
   Null,
   Void,
   Bool(bool),
   Primitive(PrimFunc),
   Other, // probably shouldnt be a thing :P
   // unimplemented types.
   // Void, Bool, Closure, Cons,
   // Int, Str, Vector, Hash, Set, Other
}

fn print_cons(f: &mut std::fmt::Formatter<'_>, car: &ScmObj, cdr: &ScmObj) -> std::fmt::Result {
   // TODO this prints an extra space in a proper list :(
   print!("{} ", car);
   match cdr {
      ScmObj::Cons(cadr, cddr) => print_cons(f, cadr, cddr),
      ScmObj::Null => write!(f, ")"),
      _ => {
         write!(f, ". ")
            .and_then(|_| write!(f, "{}", cdr)
                           .and_then(|_| write!(f, ")")))
      }
   }
}

impl std::fmt::Display for ScmObj {
   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      match *self {
         ScmObj::Numeric(n) => write!(f, "{}", n),
         ScmObj::Symbol(ref s) => write!(f, "{}", s),
         ScmObj::Null => write!(f, "'()"),
         ScmObj::Bool(true) => write!(f, "#t"),
         ScmObj::Bool(false) => write!(f, "#f"),
         ScmObj::Void => write!(f, "#<void>"),
         ScmObj::Cons(ref car, ref cdr) => {
            print!("(");
            print_cons(f, &*car, &*cdr)
         },
         ScmObj::Other => write!(f, "Other Thing! This shouldnt exist!"),
         // TODO: include prim name?
         ScmObj::Primitive(_p) => write!(f, "#<primitive>"),
      }
   }
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
         ScmObj::Cons(ref car, ref cdr) => {
            print!("(");
            print_cons(f, &*car, &*cdr)
         },
         ScmObj::Other => write!(f, "Other Thing! This shouldnt exist!"),
         // TODO: include prim name?
         ScmObj::Primitive(_p) => write!(f, "#<primitive>"),
      }
   }
}

pub struct Evaluator {
   constants: HashMap<&'static str, ScmObj>,
   // things like GC, symbol table, etc. here.
   /// the heap used at runtime for the interpreter.
   heap: arena::Arena<ScmObj>,
   /// primitive functions and functionality
   primitives: HashMap<&'static str, PrimFunc>,
   /// global symbols.
   symbols: HashMap<String, arena::Index>,
}

impl Evaluator {
   pub fn new() -> Evaluator {
      let mut constants = HashMap::new();
      constants.insert("null", ScmObj::Null);
      constants.insert("true", ScmObj::Bool(true));
      constants.insert("false", ScmObj::Bool(false));
      let primitives = prims::make_prims(); // just for code conciseness
      Evaluator {
         primitives,
         constants,
         symbols: HashMap::new(),
         heap: arena::Arena::new(),
      }
   }

   pub fn eval(&mut self, expr: ScmObj) -> () {
        self.symbols.insert("foo".into(),
                     self.heap.insert(ScmObj::Numeric(3.14)));
        let res = self.eval_inner(im::HashMap::new(), expr);
        println!("{}", res);
   }

   pub fn eval_inner(&mut self, mut locals: im::HashMap<String, arena::Index>, expr: ScmObj) -> &mut ScmObj {
      match expr {
         ScmObj::Symbol(ref s) => {
            self.fetch(&mut locals, s).expect(&*format!("Could not find symbol {:?}!", s))
         },
         ScmObj::Numeric(n) => {
            self.alloc(ScmObj::Numeric(n))
         },
         ScmObj::Cons(car, cdr) => {
            self.eval_list(locals, car, cdr)
         },
         ScmObj::Null => { self.get_const("null") },
         ScmObj::Void => { self.get_const("void") },
         // TODO: do we need to realloc this every time? idk LOL
         p@ScmObj::Primitive(_) => self.alloc(p),
         ScmObj::Bool(b) => {
            self.get_const(if b { "true" } else { "false "})
         },
         ScmObj::Other => {panic!("This type shouldnt exist! WTF DAVIS!")},
      }
   }

   fn eval_list(&mut self, mut locals: im::HashMap<String, arena::Index>, car: Box<ScmObj>, cdr: Box<ScmObj>) -> &mut ScmObj {
      match &*car {
         ScmObj::Symbol(ref s) => {
            let val = self.fetch(&mut locals, s);
            if let Some(ScmObj::Primitive(f)) = val {
               f(self, locals, *cdr)
            } else {
               // TODO: callables!!!!
               self.get_const("void")
            }
         },
         _e => {
            panic!("first element of application-list must be a callable!");
         },
      }
   }

   /// fetch a ScmObj from the constant pool
   ///   (is this called the constant pool? Or is that something else?)
   fn get_const(&mut self, name: &'static str) -> &mut ScmObj {
      self.constants.get_mut(name).expect(&*format!("ITS A CONSTANT! {:?}", name))
   }

   /// fetch a ScmObj from the locals and the symbol table.
   fn fetch(&mut self, locals: &mut im::HashMap<String, arena::Index>, name: &str) -> Option<&mut ScmObj> {
      // TODO: why dont dis wurk.
      //       shit to do with borrowing self. Would be v v nice tho.
      // locals.get(name)
      //       .or_else(|| self.symbols.get(name))
      //       .and_then(|idx| self.heap.get_mut(*idx))
      //       .or_else(|| Some(self.alloc(ScmObj::Primitive(self.primitives[name]))))

      let idx =  if locals.contains_key(name) {
         locals[name]
      } else if self.symbols.contains_key(name) {
         self.symbols[name]
      } else if self.primitives.contains_key(name) {
         return Some(self.alloc(ScmObj::Primitive(self.primitives[name])));
      } else {
         // TODO: check for a constant like #t, #f, etc.
         return None;
      };
      Some(self.heap.get_mut(idx).expect("It was in a table!"))
   }

   pub fn alloc(&mut self, obj: ScmObj) -> &mut ScmObj {
      let new_obj = self.heap.insert(obj);
      self.heap.get_mut(new_obj).expect("I JUST MADE THIS INDEX!")
   }
}

pub fn is_truthy_value(val: ScmObj) -> bool {
   if let ScmObj::Bool(false) = val {
      false
   } else {
      true
   }
}
