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
   constants: HashMap<&'static str, arena::Index>,
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
      let mut heap = arena::Arena::new();
      let mut constants = HashMap::new();
      constants.insert("null", heap.insert(ScmObj::Null));
      constants.insert("true", heap.insert(ScmObj::Bool(true)));
      constants.insert("false", heap.insert(ScmObj::Bool(false)));
      constants.insert("void", heap.insert(ScmObj::Void));
      let primitives = prims::make_prims(); // just for code conciseness
      Evaluator {
         primitives,
         constants,
         symbols: HashMap::new(),
         heap,
      }
   }

   pub fn eval(&mut self, expr: ScmObj) -> () {
      let evald_val = self.eval_inner(im::HashMap::new(), expr);
      println!("{}", self.deref_value(evald_val));
   }

   pub fn eval_inner(&mut self, mut locals: im::HashMap<String, arena::Index>, expr: ScmObj) -> arena::Index {
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

   fn eval_list(&mut self, locals: im::HashMap<String, arena::Index>, car: Box<ScmObj>, cdr: Box<ScmObj>) -> arena::Index {
      let inner_val = self.eval_inner(locals.clone(), *car);
      // take the func out of the heap, so we can have ownership.
      // THIS SEEMS REALLY BAD!
      let func = self.heap.remove(inner_val).expect("Idx doesnt exist");
      // let func = self.deref_value(inner_val);
      if let ScmObj::Primitive(pf) = func {
         // primitives are given their args unevaluated.
         pf(self, locals, *cdr)
      } else if let ScmObj::Func(formals, body) = func {
         let mut formal_params = formals.clone();
         let mut actual_params = im::HashMap::new();
         // let mut actual_params = Vec::with_capacity(formal_params.len());
         let mut head = cdr;
         loop {
            if let ScmObj::Cons(cur, next) = *head {
               if formal_params.is_empty() {
                  panic!("Too many args provided!");
               }
               let val = self.eval_inner(locals.clone(), *cur);
               actual_params.insert(formal_params.remove(0), val);
               // actual_params.push(val);
               head = next;
            } else if let ScmObj::Null = *head {
               break;
            }
         }
         if !formal_params.is_empty() {
            panic!("Not enough args provided!");
         }
         // call function!
         return self.eval_inner(actual_params.clone().union(locals), *body);
         // return self.get_const("true");
      } else {
         panic!("Only primitives currently supported.");
      }
   }

   /// fetch a ScmObj from the locals and the symbol table.
   fn fetch(&mut self, locals: &mut im::HashMap<String, arena::Index>, name: &str) -> Option<arena::Index> {
      // TODO: hmm really want something like this to work. But self.alloc needs unique access.
      // locals.get(name)
      //       .or_else(|| self.symbols.get(name))
      //       .or_else(|| Some(&self.alloc(ScmObj::Primitive(self.primitives[name]))));

      if locals.contains_key(name) {
         Some(locals[name])
      } else if self.symbols.contains_key(name) {
         Some(self.symbols[name])
      } else if self.primitives.contains_key(name) {
         return Some(self.alloc(ScmObj::Primitive(self.primitives[name])));
      } else {
         return None;
      }
   }

   /// fetch a constant
   /// TODO: this is using the same heap as everything else, it should prob be its own thing??
   //       Like constants dont need to be GCd, so it shouldnt take up heap space
   //       but we would probably need a custom GC impl, rather than using an off-the-shelf arena.
   pub fn get_const(&mut self, name: &'static str) -> arena::Index {
      *self.constants.get(name).expect(&*format!("Dont have const of name: {:?}", name))
   }

   pub fn alloc(&mut self, obj: ScmObj) -> arena::Index {
      self.heap.insert(obj)
   }

   pub fn deref_value(&mut self, idx: arena::Index) -> &mut ScmObj {
      self.heap.get_mut(idx).expect("Whoops! Idx not in the Arena!")
   }
}
