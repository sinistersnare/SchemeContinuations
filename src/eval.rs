//! The evaluator of scheme ASTs.

use crate::common::{State, SExprState, ValState, Addr, Closure, Env, Kont,
                  SExpr, Store, Time, Val, CloType, Alloc};
use crate::evaluation::eval::{expr_step};
use crate::prims::{apply_prim};
use crate::common::{val_is_list, make_scm_list,
                   scm_list_to_vals};

fn inject(ctrl: SExpr) -> SExprState {
   // Time 0 was for creation of the state, we start on 1.
   // (becuase mt is addr 0, we need to start with 1)
   SExprState::new(ctrl, Env(im::HashMap::new()), Addr(0), Time(1))
}

pub fn val_step(st: &ValState, store: &mut Store) -> State {
   let ValState {
      ctrl: val,
      env,
      kont_addr,
      ..
   } = st.clone();
   let kontval = store.get(kont_addr).expect("Dont Got Kont");
   if let Val::Kont(kont) = kontval {
      match kont {
         Kont::Empty => State::Apply(st.clone()), // fixpoint!
         Kont::If(et, ef, ifenv, next_kaddr) => {
            if val == Val::Boolean(false) {
               State::Eval(SExprState::new(ef, ifenv, next_kaddr, st.tick(1)))
            } else {
               State::Eval(SExprState::new(et, ifenv, next_kaddr, st.tick(1)))
            }
         }
         Kont::Let(vars, mut done, todo, eb, letenv, next_kaddr) => {
            done.push(val);
            if todo.is_empty() {
               let mut newenv = letenv.clone();
               for (i, (bndvar, val)) in vars.iter().zip(done.iter()).enumerate() {
                  let bnd_addr = store.add_to_store_offset(val.clone(), st, i as u64);
                  newenv = newenv.insert(bndvar.clone(), bnd_addr.clone());
               }
               State::Eval(SExprState::new(
                  eb,
                  newenv,
                  next_kaddr,
                  st.tick(1 + (vars.len() as u64)),
               ))
            } else {
               // TODO: There must be some better way to get
               // (T, Vec<T>) from Vec<T>, removing the 0th element.
               let (head, tail) = todo.split_first().unwrap();
               let new_kont = Kont::Let(vars, done, tail.to_vec(),
                                         eb, letenv.clone(), next_kaddr);
               let next_kaddr = store.add_to_store(Val::Kont(new_kont), st);
               State::Eval(SExprState::new(head.clone(), letenv, next_kaddr, st.tick(1)))
            }
         }
         Kont::Prim(op, mut done, todo, primenv, next_kaddr) => {
            done.push(val);
            if todo.is_empty() {
               let val = apply_prim(op, &done);
               State::Apply(ValState::new(val, primenv, next_kaddr, st.tick(1)))
            } else {
               let (head, tail) = todo.split_first().unwrap();
               let new_kont = Kont::Prim(op, done, tail.to_vec(), primenv.clone(), next_kaddr);
               let next_kaddr = store.add_to_store(Val::Kont(new_kont), st);
               State::Eval(SExprState::new(head.clone(), primenv, next_kaddr, st.tick(1)))
            }
         }
         Kont::ApplyPrim(op, next_kaddr) => {
            if !val_is_list(&val) { panic!("Apply not given a list.");}
            let val = apply_prim(op, &scm_list_to_vals(val));
            State::Apply(ValState::new(val, env, next_kaddr, st.tick(1)))
         }
         Kont::Callcc(next_kaddr) => {
            if let Val::Closure(Closure(clotype, body, cloenv)) = val {
               match clotype {
                  CloType::MultiArg(params) => {
                     if params.len() != 1 {
                        panic!("call/cc lambda only takes 1 argument!");
                     }
                     State::Eval(SExprState::new(
                        body,
                        cloenv.insert(params[0].clone(), next_kaddr.clone()),
                        next_kaddr,
                        st.tick(1),
                     ))
                  }
                  CloType::VarArg(_) => {
                     panic!("call/cc takes a multi-arg lambda, not vararg");
                  }
               }
            } else {
               // TODO: (call/cc k) support!
               panic!("Callcc only works with lambdas right now!");
            }
         }
         Kont::SetBang(var, next_kaddr) => {
            let addr = match env.get(var.clone()) {
               Some(v) => v,
               None => panic!("{:?} was not defined.", var),
            };
            store.set(addr, val);
            State::Apply(ValState::new(Val::Void, env, next_kaddr, st.tick(1)))
         }
         Kont::ApplyList(None, arglist, applyenv, next_kaddr) => {
            let new_kont = Kont::ApplyList(Some(Box::new(val)),
                                            arglist.clone(),
                                            applyenv.clone(),
                                            next_kaddr);
            let kont_addr = store.add_to_store(Val::Kont(new_kont), st);
            State::Eval(SExprState::new(arglist, applyenv, kont_addr, st.tick(1)))
         }
         Kont::ApplyList(Some(func), _, _, next_kaddr) => {
            if !val_is_list(&val) { panic!("Apply not given a list.");}
            if let Val::Closure(Closure(CloType::MultiArg(args), body, cloenv)) = *func {
               let mut cur = val;
               let mut argvals = Vec::with_capacity(args.len());
               while !matches!(cur, Val::Null) {
                  if let Val::Cons(car, cdr) = cur {
                     argvals.push(*car);
                     cur = *cdr;
                  } else {
                     panic!("Not given a proper list somehow.");
                  }
               }
               if argvals.len() != args.len() {
                  panic!("Mismatch arg count between func and arglist.");
               }
               let mut newenv = cloenv.clone();
               for (i, (arg, argval)) in args.iter().zip(argvals.iter()).enumerate() {
                  let argval_addr = store.add_to_store_offset(argval.clone(), st, i as u64);
                  newenv = newenv.insert(arg.clone(), argval_addr.clone());
               }
               State::Eval(SExprState::new(
                  body,
                  newenv,
                  next_kaddr,
                  st.tick(args.len() as u64)),
               )
            } else if let Val::Closure(Closure(CloType::VarArg(arg), body, cloenv)) = *func {
               let addr = store.add_to_store(val, st);
               let newenv = cloenv.insert(arg.clone(), addr);
               State::Eval(SExprState::new(body, newenv, next_kaddr, st.tick(1)))
            } else {
               panic!("Not given a function in `(apply func arglist)`");
            }
         }
         Kont::App(mut done, todo, appenv, next_kaddr) => {
            if todo.is_empty() {
               done.push(val);
               if let (Val::Closure(Closure(clotype, body, cloenv)), args) =
                  done.split_first().expect("Bad App")
               {
                  match clotype {
                     CloType::MultiArg(params) => {
                        if params.len() != args.len() {
                           panic!("arg # mismatch.");
                        }
                        let mut newenv = cloenv.clone();
                        for (i, (param, arg)) in params.iter().zip(args.iter()).enumerate() {
                           let param_addr = store.add_to_store_offset(arg.clone(), st, i as u64);
                           newenv = newenv.insert(param.clone(), param_addr.clone());
                        }
                        State::Eval(SExprState::new(
                           body.clone(),
                           newenv,
                           next_kaddr,
                           st.tick(params.len() as u64),
                        ))
                     }
                     CloType::VarArg(vararg) => {
                        let scm_list = make_scm_list(args.to_vec());
                        let addr = store.add_to_store(scm_list, st);
                        let newenv = cloenv.insert(vararg.clone(), addr);
                        State::Eval(SExprState::new(body.clone(), newenv, next_kaddr, st.tick(1)))
                     }
                  }
               } else if let (k @ Val::Kont(_), args) = done.split_first().expect("Bad CC App") {
                  if args.len() != 1 {
                     panic!("applying a kont only takes 1 argument.");
                  }

                  // replace the current continuation with the stored one.
                  let new_kaddr = store.add_to_store(k.clone(), st);
                  State::Apply(ValState::new(args[0].clone(), appenv,
                                           new_kaddr, st.tick(1)))
               } else {
                  panic!("Closure wasnt head of application");
               }
            } else {
               done.push(val);
               let (head, tail) = todo.split_first().unwrap();
               let new_kont = Kont::App(done, tail.to_vec(), appenv.clone(), next_kaddr);
               let next_kaddr = store.add_to_store(Val::Kont(new_kont), st);
               State::Eval(SExprState::new(head.clone(), appenv, next_kaddr, st.tick(1)))
            }
         }
      }
   } else {
      panic!("kont_addr not a kont addr!");
   }
}


pub fn step(st: &State, store: &mut Store) -> State {
   match st {
      State::Apply(vs) => val_step(vs, store),
      State::Eval(es) => expr_step(es, store),
   }
}

pub fn evaluate(ctrl: SExpr) -> (State, Store) {
   // initially the store only has the Empty continuation
   let mut inner = std::collections::HashMap::new();
   inner.insert(Addr(0), Val::Kont(Kont::Empty));
   let mut store = Store(inner);

   let mut st0: State = State::Eval(inject(ctrl));
   let mut stepped: State = step(&st0, &mut store);
   while st0 != stepped {
      st0 = stepped;
      stepped = step(&st0, &mut store);
   }
   (stepped, store)
}
