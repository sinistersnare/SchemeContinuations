//! The evaluator of scheme ASTs.

use std::collections::HashMap;

use generational_arena as arena;
use im;

use crate::prims::{self, prim_println};
use crate::ScmObj;

pub struct Evaluator {
   constants: HashMap<&'static str, arena::Index>,
   // things like GC, symbol table, etc. here.
   /// the heap used at runtime for the interpreter.
   pub heap: arena::Arena<ScmObj>,
   /// primitive functions and functionality
   primitives: HashMap<&'static str, arena::Index>,
   /// global symbols, things that were 'defined'.
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
      let primitives = prims::make_prims(&mut heap); // just for code conciseness
      Evaluator {
         primitives,
         constants,
         symbols: HashMap::new(),
         heap,
      }
   }

   pub fn eval(&mut self, expr: arena::Index) -> () {
      let evald_val = self.eval_inner(im::HashMap::new(), expr);
      // use the internal printing to do this.
      // but must wrap in a list first:
      let wrapped = self.cons(evald_val, self.get_const("null"));
      prim_println(self, im::HashMap::new(), wrapped);
   }

   pub fn eval_inner(
      &mut self,
      mut locals: im::HashMap<String, arena::Index>,
      expr: arena::Index,
   ) -> arena::Index {
      let obj_value = self.deref_value(expr);
      if let ScmObj::Symbol(ref s) = obj_value {
         self
            .fetch(&mut locals, s)
            .expect(&*format!("Could not find symbol {:?}!", s))
      } else if let ScmObj::Cons(car, cdr) = obj_value {
         self.eval_list(locals, *car, *cdr)
      } else {
         expr
      }
   }

   fn eval_list(
      &mut self,
      locals: im::HashMap<String, arena::Index>,
      car: arena::Index,
      cdr: arena::Index,
   ) -> arena::Index {
      let inner_val = self.eval_inner(locals.clone(), car);
      let func = self.deref_value(inner_val);
      if let ScmObj::Primitive(prim_f) = func {
         // primitives are given their args unevaluated.
         prim_f(self, locals, cdr)
      } else if let ScmObj::Func(formals, body) = func {
         self.eval_func(locals, formals.clone(), *body, cdr)
      } else {
         panic!("A callable must be in call position!");
      }
   }

   // evals a scheme-level function that is in call position
   // so ((lambda (x y) (+ x y)) 1 2) where formal_params are `[x, y]`, body is `(+ x y)`,
   // and the args list is `(cons 1 (cons 2 '()))`
   fn eval_func(
      &mut self,
      locals: im::HashMap<String, arena::Index>,
      mut formal_params: Vec<String>,
      body: arena::Index,
      args_list: arena::Index,
   ) -> arena::Index {
      let mut actual_params = im::HashMap::new();
      let mut head = args_list;
      loop {
         if let &ScmObj::Cons(cur, next) = self.deref_value(head) {
            if formal_params.is_empty() {
               panic!("Too many args provided!");
            }
            let val = self.eval_inner(locals.clone(), cur);
            actual_params.insert(formal_params.remove(0), val);
            head = next;
         } else if let ScmObj::Null = self.deref_value(head) {
            break;
         }
      }

      if !formal_params.is_empty() {
         panic!("Not enough args provided!");
      }
      // call function!
      return self.eval_inner(actual_params.clone().union(locals), body);
   }

   /// search locals, then symbols, then primitives, else return None!
   fn fetch(
      &self,
      locals: &mut im::HashMap<String, arena::Index>,
      name: &str,
   ) -> Option<arena::Index> {
      locals
         .get(name)
         .or_else(|| self.symbols.get(name))
         .or_else(|| self.primitives.get(name))
         .map(|&idx| idx)
   }

   /// fetch a constant
   /// TODO: this is using the same heap as everything else, it should prob be its own thing??
   //       Like constants dont need to be GCd, so it shouldnt take up heap space
   //       but we would probably need a custom GC impl, rather than using an off-the-shelf arena.
   pub fn get_const(&self, name: &'static str) -> arena::Index {
      *self
         .constants
         .get(name)
         .expect(&*format!("Dont have const of name: {:?}", name))
   }

   /// derefs a value from our heap to get the scheme value.
   pub fn deref_value(&self, idx: arena::Index) -> &ScmObj {
      self.heap.get(idx).expect("Whoops! Idx not in the Arena!")
   }

   pub fn add_symbol(&mut self, name: String, value: arena::Index) {
      self.symbols.insert(name, value);
   }

   /// The allocation function of this interpreter.
   /// takes an object and puts it on our heap!
   pub fn alloc(&mut self, obj: ScmObj) -> arena::Index {
      self.heap.insert(obj)
   }

   /// a helper that allocates a cons object for us.
   /// it may be best to do all allocating in 1 function
   /// so we can see that it only ever happens when we use 'alloc'.
   /// but this seemed like a nice shortcut :)
   pub fn cons(&mut self, car: arena::Index, cdr: arena::Index) -> arena::Index {
      self.alloc(ScmObj::Cons(car, cdr))
   }
}
