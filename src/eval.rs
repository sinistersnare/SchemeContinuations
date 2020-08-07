//! The evaluator of scheme ASTs.

use std::collections::HashMap;

use im;
use generational_arena as arena;

use crate::prims::{self, PrimFunc};
use crate::ScmObj;

struct StackFrame {
   args: Vec<ScmObj>,
   locals: HashMap<String, ScmObj>,
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
      constants.insert("void", ScmObj::Void);
      let primitives = prims::make_prims(); // just for code conciseness
      Evaluator {
         primitives,
         constants,
         symbols: HashMap::new(),
         heap: arena::Arena::new(),
      }
   }

   pub fn eval(&mut self, expr: ScmObj) -> () {
      // TODO remove this after we have variable binding.
      self.symbols.insert("foo".into(),
                           self.heap.insert(ScmObj::Numeric(3.14)));
      println!("{}", self.eval_inner(im::HashMap::new(), expr));
   }

   pub fn eval_inner(&mut self, mut locals: im::HashMap<String, arena::Index>, expr: ScmObj) -> &mut ScmObj {
      match expr {
         ScmObj::Symbol(ref s) => self.fetch(&mut locals, s).expect(&*format!("Could not find symbol {:?}!", s)),
         ScmObj::Numeric(n) => self.alloc(ScmObj::Numeric(n)),
         ScmObj::Cons(car, cdr) => self.eval_list(locals, car, cdr),
         ScmObj::Null => self.get_const("null"),
         ScmObj::Void => self.get_const("void"),
         // TODO: do we need to realloc these every time? idk LOL
         p@ScmObj::Primitive(_) => self.alloc(p),
         f@ScmObj::Func(..) => self.alloc(f),
         ScmObj::Bool(b) => self.get_const(if b { "true" } else { "false"}),
         ScmObj::Other => panic!("This type shouldnt exist! WTF DAVIS!"),
      }
   }

   fn eval_list(&mut self, locals: im::HashMap<String, arena::Index>, car: Box<ScmObj>, cdr: Box<ScmObj>) -> &mut ScmObj {
      let func = self.eval_inner(locals.clone(), *car);
      if let ScmObj::Primitive(pf) = func {
         pf(self, locals, *cdr)
      } else if let ScmObj::Func(_formal_params, _body) = func {
         // let mut actual_params = Vec::with_capacity(formal_params.len());
         // let mut head = cdr;
         // loop {
         //    if let ScmObj::Cons(cur, next) = *head {
         //       // let val = self.eval_inner(locals.clone(), *cur);
         //       // actual_params.push(val);
         //       // head = next;
         //       // println!("head: {:?}", head);
         //    } else if let ScmObj::Null = *head {
         //       break;
         //    }
         // }
         return self.get_const("void");
      } else {
         panic!("Only primitives currently supported.");
      }
   }

   /// fetch a ScmObj from the locals and the symbol table.
   fn fetch(&mut self, locals: &mut im::HashMap<String, arena::Index>, name: &str) -> Option<&mut ScmObj> {
      // TODO: why dont dis wurk.
      //       shit to do with borrowing self. Would be v v nice tho.
      // locals.get(name)
      //       .or_else(|| self.symbols.get(name))
      //       .and_then(|idx| self.heap.get_mut(*idx)) // should work if an idx exists.
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

   /// fetch a ScmObj from the constant pool
   ///   (is this called the constant pool? Or is that something else?)
   pub fn get_const(&mut self, name: &'static str) -> &mut ScmObj {
      self.constants.get_mut(name).expect(&*format!("ITS A CONSTANT! {:?}", name))
   }

   pub fn alloc(&mut self, obj: ScmObj) -> &mut ScmObj {
      let new_obj = self.heap.insert(obj);
      self.heap.get_mut(new_obj).expect("I JUST MADE THIS INDEX!")
   }
}

pub fn is_truthy_value(val: &mut ScmObj) -> bool {
   if let ScmObj::Bool(false) = val {
      false
   } else {
      true
   }
}
