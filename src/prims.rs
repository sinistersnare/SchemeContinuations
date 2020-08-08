//! Prims take their arguments unevaluated and decides what to do with them
//! themselves. For example, `void` does not evaluate its arguments at all,
//! `if` will not evaluate the false branch if the true is taken (and vice versa).
//! So, they must evaluate their args them if they want to use them.
//!
//! This isnt 'correct' IMO becuase you cant do (apply lambda foo),
//! because lambda isnt a primitive function, its a special form!
//! But im lazy and this should work for now.

use std::collections::HashMap;

use im;
use generational_arena as arena;

use crate::eval::{Evaluator};
use crate::{ScmObj, is_truthy_value};

pub type PrimFunc = fn(&mut Evaluator, im::HashMap<String, arena::Index>, arena::Index) -> arena::Index;

pub fn make_prims(heap: &mut arena::Arena<ScmObj>) -> HashMap<&'static str, arena::Index> {
   // TODO: Use a perfect hash function map! Or Something like that!
   //       or use a hashmap size/load_factor so it never has to grow more.
   let mut map = HashMap::new();
   map.insert("if", heap.insert(ScmObj::Primitive(prim_if)));
   map.insert("+", heap.insert(ScmObj::Primitive(prim_plus)));
   map.insert("*", heap.insert(ScmObj::Primitive(prim_mul)));
   map.insert("cons", heap.insert(ScmObj::Primitive(prim_cons)));
   map.insert("quote", heap.insert(ScmObj::Primitive(prim_quote)));
   map.insert("begin", heap.insert(ScmObj::Primitive(prim_begin)));
   map.insert("not", heap.insert(ScmObj::Primitive(prim_not)));
   map.insert("define", heap.insert(ScmObj::Primitive(prim_define)));
   map.insert("lambda", heap.insert(ScmObj::Primitive(prim_lambda)));
   // returns a void value, all arguments are ignored and not evaluated.
   map.insert("void", heap.insert(ScmObj::Primitive(|eval, _locals, _args| eval.get_const("void"))));
   map.insert("void?", heap.insert(ScmObj::Primitive(prim_void_huh)));
   map.insert("let", heap.insert(ScmObj::Primitive(prim_let)));
   map.insert("println", heap.insert(ScmObj::Primitive(prim_println)));
   map
}

fn prim_if(ctx: &mut Evaluator, locals: im::HashMap<String, arena::Index>, args: arena::Index) -> arena::Index {
   if let &ScmObj::Cons(cond_part, then_else_rest) = ctx.deref_value(args) {
      if let &ScmObj::Cons(then_part, else_rest) = ctx.deref_value(then_else_rest) {
         if let &ScmObj::Cons(else_part, null_part) = ctx.deref_value(else_rest) {
            if let &ScmObj::Null = ctx.deref_value(null_part) {
               let cond_val = ctx.eval_inner(locals.clone(), cond_part);
               if is_truthy_value(ctx.deref_value(cond_val)) {
                  ctx.eval_inner(locals, then_part)
               } else {
                  ctx.eval_inner(locals, else_part)
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
fn prim_plus(ctx: &mut Evaluator, locals: im::HashMap<String, arena::Index>, args: arena::Index) -> arena::Index {
   let mut cur = args;
   let mut sum: f64 = 0.0;
   loop {
      match ctx.deref_value(cur) {
         &ScmObj::Cons(car, cdr) => {
            let num_val = ctx.eval_inner(locals.clone(), car);
            if let ScmObj::Numeric(n) = *ctx.deref_value(num_val) {
               sum += n;
               cur = cdr;
            } else {
               panic!("Only numbers can be added!");
            }
         },
         ScmObj::Null => { return ctx.alloc(ScmObj::Numeric(sum)); },
         _ => { panic!("Only numbers can be added!")}
      }
   }
}

/// Takes any number of arguments in a proper list, and returns the product of them.
/// if any of the args are not numbers, then this will fail.
fn prim_mul(ctx: &mut Evaluator, locals: im::HashMap<String, arena::Index>, args: arena::Index) -> arena::Index {
   let mut cur = args;
   let mut sum: f64 = 1.0;
   loop {
      match ctx.deref_value(cur) {
         &ScmObj::Cons(car, cdr) => {
            let num_val = ctx.eval_inner(locals.clone(), car);
            if let ScmObj::Numeric(n) = *ctx.deref_value(num_val) {
               sum *= n;
               cur = cdr;
            } else {
               panic!("Only numbers can be multiplied!");
            }
         },
         ScmObj::Null => { return ctx.alloc(ScmObj::Numeric(sum)); },
         _ => { panic!("Only numbers can be multiplied!")}
      }
   }
}

/// (quote 5) => (quote 5). Doesnt do anything!
/// but (eval (quote 5)) does something. Hmmm how does that work!
/// I think eval checks if its quoted and just removes that?
fn prim_quote(ctx: &mut Evaluator, _: im::HashMap<String, arena::Index>, args: arena::Index) -> arena::Index {
   // rust isnt smart enough to let me put these together!
   let quote = ctx.alloc(ScmObj::Symbol("quote".to_string()));
   let nil = ctx.get_const("null");
   let end = ctx.cons(args, nil);
   ctx.cons(quote, end)
}

fn prim_cons(ctx: &mut Evaluator, locals: im::HashMap<String, arena::Index>, args: arena::Index) -> arena::Index {
   if let &ScmObj::Cons(car, cdr) = ctx.deref_value(args) {
      if let &ScmObj::Cons(cadr, cddr) = ctx.deref_value(cdr) {
         if let &ScmObj::Null = ctx.deref_value(cddr) {
            let quote = ctx.alloc(ScmObj::Symbol("quote".to_string()));
            let eval_car = ctx.eval_inner(locals.clone(), car);
            let eval_cadr = ctx.eval_inner(locals, cadr);
            let cons = ctx.cons(eval_car, eval_cadr);
            ctx.cons(quote, cons)
         } else {
            panic!("cons only takes 2 args.");
         }
      } else {
         panic!("cons is supposed to take 2 args.");
      }
   } else {
      panic!("Cons not given a list as args.");
   }
}

fn prim_void_huh(ctx: &mut Evaluator, locals: im::HashMap<String, arena::Index>, args: arena::Index) -> arena::Index {
   // this function only allows a 1-length list.
   if let &ScmObj::Cons(car, cdr) = ctx.deref_value(args) {
      if let &ScmObj::Null = ctx.deref_value(cdr) {
         let void_val = ctx.eval_inner(locals, car);
         if let ScmObj::Void = ctx.deref_value(void_val) {
            ctx.get_const("true")
         } else {
            ctx.get_const("false")
         }
      } else {
         panic!("Only a length-1 list allowed in `void?`.");
      }
   } else {
      panic!("`void?` not given a list as args.");
   }
}

/// evaluates each argument, and returns the last one.
/// if no args are provided, void is returned. This differs from racket semantics.
fn prim_begin(ctx: &mut Evaluator, locals: im::HashMap<String, arena::Index>, args: arena::Index) -> arena::Index {
   let mut latest = args;
   loop {
      if let &ScmObj::Cons(car, cdr) = ctx.deref_value(latest) {
         // must replicate the eval_inner due to some lifetime shit.
         if let ScmObj::Null = ctx.deref_value(cdr) {
            return ctx.eval_inner(locals.clone(), car);
         } else {
            ctx.eval_inner(locals.clone(), car);
            latest = cdr;
         }
      } else if let ScmObj::Null = ctx.deref_value(latest) {
         return ctx.get_const("void");
      } else {
         panic!("Args must be a proper list!");
      }
   }
}

/// TODO: this can be impld as a regular function, not a primitive.
fn prim_not(ctx: &mut Evaluator, locals: im::HashMap<String, arena::Index>, args: arena::Index) -> arena::Index {
   if let ScmObj::Cons(car, cdr) = ctx.deref_value(args) {
      if let ScmObj::Null = ctx.deref_value(*cdr) {
         let bool_val = ctx.eval_inner(locals, *car);
         if let ScmObj::Bool(b) = ctx.deref_value(bool_val) {
            // ctx.get_const(if *b { "true" } else { "false" })
            if *b {
               ctx.get_const("true")
            } else {
               ctx.get_const("false")
            }
         } else {
            panic!("Arg to `not` must be a bool.");
         }
      } else {
         panic!("`not` takes only 1 argument.");
      }
   } else {
      panic!("Args must be a proper list!");
   }
}

// right now only allows the form (define a b) not the function-define style of (define (f a b) body).
fn prim_define(ctx: &mut Evaluator, locals: im::HashMap<String, arena::Index>, args: arena::Index) -> arena::Index {
   if let &ScmObj::Cons(name, val_rest) = ctx.deref_value(args) {
      if let &ScmObj::Cons(val, null_part) = ctx.deref_value(val_rest) {
         if let ScmObj::Null = ctx.deref_value(null_part) {
            // stolen from elsewhere
            let is_sym = {
               if let ScmObj::Symbol(s) = ctx.deref_value(name) {
                  Some(s.clone())
               } else {
                  None
               }
            };
            if let Some(s) = is_sym {
               let evald_val = ctx.eval_inner(locals, val);
               ctx.add_symbol(s, evald_val);
               ctx.get_const("void")
            } else {
               panic!("Define takes a bare symbol as first argument.");
            }
         } else {
            panic!("define only takes exactly 2 arguments");
         }
      } else {
         panic!("Must provide a 2nd argument");
      }
   } else {
      panic!("A list must be given as arguments!");
   }
}

/// Returns a lambda object, doesnt use locals for anything, I dont think it should.
fn prim_lambda(ctx: &mut Evaluator, _: im::HashMap<String, arena::Index>, args: arena::Index) -> arena::Index {
   if let ScmObj::Cons(formals_obj, body_rest) = ctx.deref_value(args) {
      // add all the formals into a vec
      let mut formal_names = Vec::new();
      let mut cur = formals_obj;
      loop {
         if let ScmObj::Cons(formal, rest) = ctx.deref_value(*cur) {
            if let ScmObj::Symbol(s) = ctx.deref_value(*formal) {
               formal_names.push(s.clone());
            } else {
               panic!("The list of formal parameters must be symbols!");
            }
            cur = rest;
         } else if let ScmObj::Null = ctx.deref_value(*cur) {
            break;
         } else {
            panic!("The formal list must be a proper list!");
         }
      }
      // now ensure the body is a single ScmObj.
      if let ScmObj::Cons(body, rest) = ctx.deref_value(*body_rest) {
         if let ScmObj::Null = ctx.deref_value(*rest) {
            // dont eval the body, that gets evald later!!! Somehow!!
            // but DO store it in the heap for safekeeping...
            ctx.alloc(ScmObj::Func(formal_names, *body))
         } else {
            panic!("Lambda body allows only 1 expression.");
         }
      } else {
         panic!("lambda must take a body.");
      }
   } else {
      panic!("Was not given a list of arguments.");
   }
}

/// takes something of form (let ((a 1) (c 2)) body)
/// and evaluates body after adding a and c to the local environment.
fn prim_let(ctx: &mut Evaluator, locals: im::HashMap<String, arena::Index>, args: arena::Index) -> arena::Index {
   if let &ScmObj::Cons(bindings, body_rest) = ctx.deref_value(args) {
      let mut new_bindings = im::HashMap::new();
      let mut cur = bindings;
      loop {
         if let &ScmObj::Cons(single_binding, rest_bindings) = ctx.deref_value(cur) {
            if let &ScmObj::Cons(binding_name, value_rest) = ctx.deref_value(single_binding) {
               if let &ScmObj::Cons(value, null_part) = ctx.deref_value(value_rest) {
                  if let ScmObj::Null = ctx.deref_value(null_part) {
                     // have to mess with the lifetimes a bit to make sure the
                     // deref'd value doesnt live too long.
                     // Rust isnt _that_ smart I guess :( cant write the prettiest code.
                     let is_sym = {
                        if let ScmObj::Symbol(s) = ctx.deref_value(binding_name) {
                           Some(s.clone())
                        } else {
                           None
                        }
                     };
                     if let Some(s) = is_sym {
                        let binding_value = ctx.eval_inner(locals.clone(), value);
                        new_bindings.insert(s, binding_value);
                     } else {
                        panic!("Binding names must be symbols.");
                     }
                     cur = rest_bindings;
                  } else {
                     panic!("let binding takes only 1 value.");
                  }
               } else {
                  panic!("improper formed let binding");
               }
            } else {
               panic!("Improper formed let binding");
            }
         } else if let ScmObj::Null = ctx.deref_value(cur) {
            break;
         } else {
            panic!("Idk what happened here!");
         }
      }
      if let ScmObj::Cons(body, null_part) = ctx.deref_value(body_rest) {
         if let ScmObj::Null = ctx.deref_value(*null_part) {
            ctx.eval_inner(new_bindings.union(locals.clone()), *body)
         } else {
            panic!("Only 1 expression allowed in let binding body.");
         }
      } else {
         panic!("Body was wrong!!");
      }
   } else {
      panic!("Args must be a list.");
   }
}

pub fn prim_println(ctx: &mut Evaluator, locals: im::HashMap<String, arena::Index>, args: arena::Index) -> arena::Index {
   prim_print(ctx, locals, args);
   println!();
   ctx.get_const("void")
}

fn prim_print(ctx: &mut Evaluator, locals: im::HashMap<String, arena::Index>, args: arena::Index) -> arena::Index {
   if let &ScmObj::Cons(obj, null_part) = ctx.deref_value(args) {
      if let &ScmObj::Null = ctx.deref_value(null_part) {
         let evald = ctx.eval_inner(locals.clone(), obj);
         print_aux(ctx, locals, evald);
      } else {
         panic!("Only 1 arg to println allowed");
      }
   } else {
      panic!("println not given a list as an argument???");
   }
   ctx.get_const("void")
}

// this is the object itself that we are printing, not the arg list we are receiving.
fn print_aux(ctx: &mut Evaluator, locals: im::HashMap<String, arena::Index>, obj: arena::Index) {
   match *ctx.deref_value(obj) {
      ScmObj::Numeric(n) => print!("{}", n),
      ScmObj::Symbol(ref s) => print!("{}", s),
      ScmObj::Null => print!("'()"),
      ScmObj::Bool(true) => print!("#t"),
      ScmObj::Bool(false) => print!("#f"),
      ScmObj::Void => print!("#<void>"),
      ScmObj::Cons(car, cdr) => {
         print!("(");
         prim_printcons(ctx, locals, car, cdr)
      },
      ScmObj::Func(..) => print!("#<function>"),
      ScmObj::Other => print!("Other Thing! This shouldnt exist!"),
      // TODO: include prim name somehow?
      ScmObj::Primitive(_p) => print!("#<primitive>"),
   }
}

// a helper function for printing lists.
fn prim_printcons(ctx: &mut Evaluator, locals: im::HashMap<String, arena::Index>, car: arena::Index, cdr: arena::Index) {
   print_aux(ctx, locals.clone(), car);
   print!(" ");
   match ctx.deref_value(cdr) {
      &ScmObj::Cons(cadr, cddr) => {
         prim_printcons(ctx, locals, cadr, cddr)
      },
      // FIXME: THIS IS FUCKING FUCK UGLY!!!!!!!!!!!!!!!
      // DAVIS YOU FUCKER
      // YOU SHOULDNT USE ESCAPE SEQUENCES DAVIS
      // but lifetimes are hard :(
      // FUCK YOU
      &ScmObj::Null => {
         // write a backspace ascii code to the formatter
         // because im not smart enough to get around
         // lifetime stuff I guess.
         print!("{}", (8u8 as char));
         print!(")")},
      _ => {
         // improper list ending.
         print!(". ");
         print_aux(ctx, locals, cdr);
         print!(")")
      }
   }
}


