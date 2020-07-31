
use std::collections::HashMap;

use im;
use generational_arena as arena;

use crate::eval::{ScmObj, Evaluator, is_truthy_value};

pub type PrimFunc = fn(&mut Evaluator, im::HashMap<String, arena::Index>, ScmObj) -> &mut ScmObj;

pub fn make_prims() -> HashMap<&'static str, PrimFunc> {
   // for some reason I need to explicitly annotate this!
   // idk why the signature isnt enough.
   let mut map: HashMap<_, PrimFunc> = HashMap::new();
   map.insert("if", prim_if);
   map.insert("+", prim_plus);
   map.insert("quote", prim_quote);
   map
}

fn prim_if(ctx: &mut Evaluator, locals: im::HashMap<String, arena::Index>, args: ScmObj) -> &mut ScmObj {
   if let ScmObj::Cons(cond_part, then_else_rest) = args {
      if let ScmObj::Cons(then_part, else_rest) = *then_else_rest {
         if let ScmObj::Cons(else_part, null_part) = *else_rest {
            if let ScmObj::Null = *null_part {
               if is_truthy_value(*cond_part) {
                  ctx.eval_inner(locals, *then_part)
               } else {
                  ctx.eval_inner(locals, *else_part)
               }
            } else {
               panic!("`if` form requires 3 parts!")
            }
         } else {
            panic!("`if` form requires 3 parts!")
         }
      } else {
         panic!("`if` form requires 3 parts!")
      }
   } else {
      panic!("`if` form requires 3 parts!")
   }
}

/// Takes any number of arguments in a proper list, and returns the sum of them.
/// if any of the args are not numbers, then this will fail.
fn prim_plus(ctx: &mut Evaluator, _: im::HashMap<String, arena::Index>, args: ScmObj) -> &mut ScmObj {
   let mut cur = args;
   let mut sum = 0.0;
   loop {
      match cur {
         ScmObj::Cons(car, cdr) => {
            if let ScmObj::Numeric(n) = *car {
               sum += n;
               cur = *cdr;
            } else {
               panic!("Only numbers can be added!");
            }
         },
         ScmObj::Null => { return ctx.alloc(ScmObj::Numeric(sum)); },
         _ => { panic!("Only numbers can be added!")}
      }
   }
}

/// (quote 5) => (quote 5). Doesnt do anything!
/// but (eval (quote 5)) does something. Hmmm how does that work!
fn prim_quote(ctx: &mut Evaluator, _: im::HashMap<String, arena::Index>, args: ScmObj) -> &mut ScmObj {
   ctx.alloc(ScmObj::Cons(Box::new(ScmObj::Symbol("quote".to_string())),
                          Box::new(ScmObj::Cons(Box::new(args),
                                                Box::new(ScmObj::Null)))))
}
