//! The evaluator of scheme ASTs.

use crate::common::{Addr, Closure, Env, Kont, Prim, SExpr,
                    State, Store, Time, Val, Var, CloType};
use crate::prims::PRIMS;

fn inject(ctrl: SExpr) -> State {
   // Time 0 was for creation of the state, we start on 1.
   // (becuase mt is addr 0, we need to start with 1)
   State::new(ctrl, Env(im::HashMap::new()), Addr(0), Time(1))
}

fn val_is_list(val: &Val) -> bool {
   if !matches!(val, Val::Cons(_, _)|Val::Null) {
      return false;
   }
   let mut cur = val;
   while let Val::Cons(_, cdr) = cur {
      cur = &*cdr;
   }
   matches!(cur, Val::Null)
}

fn make_scm_list(vals: Vec<Val>) -> Val {
   let mut lst = Val::Null;
   for v in vals.into_iter().rev() {
      lst = Val::Cons(Box::new(v), Box::new(lst));
   }
   lst
}

fn scm_list_to_vals(val: Val) -> Vec<Val> {
   let mut vals = vec![];
   let mut cur = val;
   while let Val::Cons(car, cdr) = cur {
      vals.push(*car);
      cur = *cdr;
   }
   vals
}

fn is_atomic(ctrl: &SExpr) -> bool {
   match ctrl {
      SExpr::List(ref list) => matches!(matches_lambda_expr(list), Some(_)),
      SExpr::Atom(_) => true,
   }
}

fn matches_number(str: &str) -> Option<i64> {
   str.parse::<i64>().ok()
}

fn matches_boolean(str: &str) -> Option<bool> {
   // because we cant parse #t/#f rn, just use true/false.
   if str == "true" {
      Some(true)
   } else if str == "false" {
      Some(false)
   } else {
      None
   }
}

fn matches_apply_expr(list:  &[SExpr]) -> Option<(SExpr, SExpr)> {
   if list.len() == 3 && list[0] == SExpr::Atom("apply".to_string()) {
      Some((list[1].clone(), list[2].clone()))
   } else {
      None
   }
}

fn matches_lambda_expr(list: &[SExpr]) -> Option<(CloType, SExpr)> {
   if list.len() == 3
      && (list[0] == SExpr::Atom("lambda".to_string()) || list[0] == SExpr::Atom("λ".to_string()))
   {
      match list[1] {
         SExpr::List(ref args) => {
            let mut argvec = Vec::with_capacity(args.len());
            for arg_sexpr in args {
               match arg_sexpr {
                  SExpr::List(_) => {
                     panic!("Unexpected list in argument position");
                  }
                  SExpr::Atom(ref arg) => {
                     argvec.push(Var(arg.clone()));
                  }
               };
            }
            Some((CloType::MultiArg(argvec), list[2].clone()))
         }
         SExpr::Atom(ref var) => {
            Some((CloType::VarArg(Var(var.clone())), list[2].clone()))
         }
      }
   } else {
      None
   }
}

fn matches_if_expr(list: &[SExpr]) -> Option<(SExpr, SExpr, SExpr)> {
   if list.len() == 4 && list[0] == SExpr::Atom("if".to_string()) {
      Some((list[1].clone(), list[2].clone(), list[3].clone()))
   } else {
      None
   }
}

fn matches_let_expr(list: &[SExpr]) -> Option<(Vec<Var>, Vec<SExpr>, SExpr)> {
   if list.len() == 3 && list[0] == SExpr::Atom("let".to_string()) {
      match list[1] {
         SExpr::List(ref outer) => {
            let mut vars = Vec::with_capacity(outer.len());
            let mut exprs = Vec::with_capacity(outer.len());
            for binding in outer {
               match binding {
                  SExpr::List(ref entry) => {
                     if entry.len() != 2 {
                        panic!("Let entry must only have 2 elements.");
                     }
                     match entry[0] {
                        SExpr::List(_) => {
                           panic!("Binding name must be an atom.");
                        }
                        SExpr::Atom(ref v) => {
                           vars.push(Var(v.clone()));
                           exprs.push(entry[1].clone());
                        }
                     }
                  }
                  SExpr::Atom(_) => {
                     panic!("Bindings are len-2 lists, not atoms.");
                  }
               }
            }
            Some((vars, exprs, list[2].clone()))
         }
         SExpr::Atom(_) => {
            panic!("Let takes a binding list, not a single arg");
         }
      }
   } else {
      None
   }
}

fn matches_prim_expr(list: &[SExpr]) -> Option<(Prim, SExpr, Vec<SExpr>)> {
   if list.len() >= 3 && list[0] == SExpr::Atom("prim".to_string()) {
      let primname = match list[1] {
         SExpr::List(_) => {
            panic!("Unexpected list in prim-name position");
         }
         SExpr::Atom(ref name) => name.clone(),
      };
      let (left, right) = list.split_at(3);
      let arg0 = left[2].clone();
      Some((Prim(primname), arg0, right.to_vec()))
   } else {
      None
   }
}

fn matches_apply_prim_expr(list: &[SExpr]) -> Option<(Prim, SExpr)> {
   if list.len() == 3 && list[0] == SExpr::Atom("apply-prim".to_string()) {
      let primname = match list[1] {
         SExpr::List(_) => {
            panic!("Unexpected list in prim-name position");
         }
         SExpr::Atom(ref name) => name.clone(),
      };
      let arglist = list[2].clone();
      Some((Prim(primname), arglist))
   } else {
      None
   }
}

fn matches_callcc_expr(list: &[SExpr]) -> Option<SExpr> {
   if list.len() == 2 && list[0] == SExpr::Atom("call/cc".to_string()) {
      Some(list[1].clone())
   } else {
      None
   }
}

fn matches_setbang_expr(list: &[SExpr]) -> Option<(Var, SExpr)> {
   if list.len() == 3 && list[0] == SExpr::Atom("set!".to_string()) {
      match list[1] {
         SExpr::Atom(ref x) => {
            Some((Var(x.clone()), list[2].clone()))
         }
         SExpr::List(_) => {
            panic!("set! takes a var then an expression");
         }
      }
   } else {
      None
   }
}

fn apply_prim(Prim(op): Prim, args: &[Val]) -> Val {
   (*PRIMS.get::<str>(&op).expect("Prim doesnt exist!"))(args)
}

fn atomic_eval(ctrl: &SExpr, env: &Env, store: &Store) -> Val {
   match ctrl {
      SExpr::List(ref list) => {
         if let Some((args, body)) = matches_lambda_expr(list) {
            Val::Closure(Closure(args, body, env.clone()))
         } else {
            panic!("Not given an atomic value, given some list.");
         }
      }
      SExpr::Atom(ref atom) => {
         if let Some(n) = matches_number(atom) {
            Val::Number(n)
         } else if let Some(b) = matches_boolean(atom) {
            Val::Boolean(b)
         } else {
            store
               .get(env.get(Var(atom.clone())).expect("Atom not in env"))
               .expect("Atom not in store")
         }
      }
   }
}

fn step_atomic(st: &State, store: &mut Store) -> State {
   let State {
      ctrl,
      env,
      kont_addr,
      ..
   } = st.clone();
   let val = atomic_eval(&ctrl, &env, &store);
   let kontval = store.get(kont_addr).expect("Dont Got Kont");
   if let Val::Kont(kont) = kontval {
      match kont {
         Kont::Empty => st.clone(), // fixpoint!
         Kont::If(et, ef, ifenv, next_kaddr) => {
            if val == Val::Boolean(false) {
               State::new(ef, ifenv, next_kaddr, st.tick(1))
            } else {
               State::new(et, ifenv, next_kaddr, st.tick(1))
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
               State::new(
                  eb,
                  newenv,
                  next_kaddr,
                  st.tick(1 + (vars.len() as u64)),
               )
            } else {
               // TODO: There must be some better way to get
               // (T, Vec<T>) from Vec<T>, removing the 0th element.
               let (head, tail) = todo.split_first().unwrap();
               let new_kont = Kont::Let(vars, done, tail.to_vec(),
                                         eb, letenv.clone(), next_kaddr);
               let next_kaddr = store.add_to_store(Val::Kont(new_kont), st);
               State::new(head.clone(), letenv, next_kaddr, st.tick(1))
            }
         }
         Kont::Prim(op, mut done, todo, primenv, next_kaddr) => {
            done.push(val);
            if todo.is_empty() {
               let val = apply_prim(op, &done);
               // TODO: UNFUCK THIS (see TODO under this)
               //       we cant have nice things when like this.
               let ae = if let Val::Number(n) = val {
                  n.to_string()
               } else if let Val::Boolean(true) = val {
                  "true".to_string()
               } else if let Val::Boolean(false) = val {
                  "false".to_string()
               } else {
                  panic!("Prims right now can only return numbers and booleans. Sorry!");
               };
               // TODO: we shouldnt be downgrading to AE here.
               // but fuck if I know how to fix it.
               State::new(SExpr::Atom(ae), primenv, next_kaddr, st.tick(1))
            } else {
               let (head, tail) = todo.split_first().unwrap();
               let new_kont = Kont::Prim(op, done, tail.to_vec(), primenv.clone(), next_kaddr);
               let next_kaddr = store.add_to_store(Val::Kont(new_kont), st);
               State::new(head.clone(), primenv, next_kaddr, st.tick(1))
            }
         }
         Kont::ApplyPrim(op, next_kaddr) => {
            if !val_is_list(&val) { panic!("Apply not given a list.");}
            let val = apply_prim(op, &scm_list_to_vals(val));
            // TODO: UNFUCK THIS (see TODO under this)
            //       we cant have nice things when like this.
            let ae = if let Val::Number(n) = val {
               n.to_string()
            } else if let Val::Boolean(true) = val {
               "true".to_string()
            } else if let Val::Boolean(false) = val {
               "false".to_string()
            } else {
               panic!("Prims right now can only return numbers and booleans. Sorry!");
            };
            // TODO: we shouldnt be downgrading to AE here.
            // but fuck if I know how to fix it.
            State::new(SExpr::Atom(ae), env, next_kaddr, st.tick(1))
         }
         Kont::Callcc(next_kaddr) => {
            if let Val::Closure(Closure(clotype, body, cloenv)) = val {
               match clotype {
                  CloType::MultiArg(params) => {
                     if params.len() != 1 {
                        panic!("call/cc lambda only takes 1 argument!");
                     }
                     State::new(
                        body,
                        cloenv.insert(params[0].clone(), next_kaddr.clone()),
                        next_kaddr,
                        st.tick(1),
                     )
                  }
                  CloType::VarArg(_) => {
                     panic!("call/cc takes a multi-arg lambda, not vararg");
                  }
               }
            } else {
               panic!("Callcc only works with lambdas right now! TODO: (call/cc k) support!");
            }
         }
         Kont::SetBang(var, next_kaddr) => {
            let addr = match env.get(var.clone()) {
               Some(v) => v,
               None => panic!("{:?} was not defined.", var),
            };
            store.set(addr, val);
            // TODO CANT RETURN VALUE HERE DAVIS UGH
            //    SHOULD BE RETURNING VOID HERE.
            State::new(SExpr::Atom("-42".to_string()), env,
                       next_kaddr, st.tick(1))
         }
         Kont::ApplyList(None, arglist, applyenv, next_kaddr) => {
            let new_kont = Kont::ApplyList(Some(Box::new(val)),
                                            SExpr::Atom("UNUSED!".into()),
                                            applyenv.clone(), next_kaddr);
            let kont_addr = store.add_to_store(Val::Kont(new_kont), st);
            State::new(arglist, applyenv, kont_addr, st.tick(1))
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
               State::new(
                  body,
                  newenv,
                  next_kaddr,
                  st.tick(args.len() as u64),
               )
            } else if let Val::Closure(Closure(CloType::VarArg(arg), body, cloenv)) = *func {
               let addr = store.add_to_store(val, st);
               let newenv = cloenv.insert(arg.clone(), addr);
               State::new(body, newenv, next_kaddr, st.tick(1))
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
                        State::new(
                           body.clone(),
                           newenv,
                           next_kaddr,
                           st.tick(params.len() as u64),
                        )
                     }
                     CloType::VarArg(vararg) => {
                        let scm_list = make_scm_list(args.to_vec());
                        let addr = store.add_to_store(scm_list, st);
                        let newenv = cloenv.insert(vararg.clone(), addr);
                        State::new(body.clone(), newenv, next_kaddr, st.tick(1))
                     }
                  }
               } else if let (k @ Val::Kont(_), args) = done.split_first().expect("Bad CC App") {
                  if args.len() != 1 {
                     panic!("applying a kont only takes 1 argument.");
                  }

                  // replace the current continuation with the stored one.
                  let new_kaddr = store.add_to_store(k.clone(), st);
                  State::new(ctrl, appenv, new_kaddr, st.tick(1))
               } else {
                  panic!("Closure wasnt head of application");
               }
            } else {
               done.push(val);
               let (head, tail) = todo.split_first().unwrap();
               let new_kont = Kont::App(done, tail.to_vec(), appenv.clone(), next_kaddr);
               let next_kaddr = store.add_to_store(Val::Kont(new_kont), st);
               State::new(head.clone(), appenv, next_kaddr, st.tick(1))
            }
         }
      }
   } else {
      panic!("kont_addr not a kont addr!");
   }
}

fn step(st: &State, store: &mut Store) -> State {
   let State {
      ctrl,
      env,
      kont_addr,
      ..
   } = st.clone();
   if is_atomic(&ctrl) {
      step_atomic(st, store)
   } else {
      match ctrl {
         SExpr::List(ref list) => {
            if let Some((ec, et, ef)) = matches_if_expr(list) {
               let new_kont = Kont::If(et, ef, env.clone(), kont_addr);
               let next_kaddr = store.add_to_store(Val::Kont(new_kont), st);
               State::new(ec, env, next_kaddr, st.tick(1))
            } else if let Some((vars, exprs, eb)) = matches_let_expr(list) {
               let len = vars.len();
               if len == 0 {
                  // why would you write (let () eb) you heathen
                  // because of you I have to cover this case
                  // TODO: make sure this is covered in the formalization
                  //       And this is how it should be done for prims too
                  //       I think....
                  State::new(eb, env, kont_addr, st.tick(1))
               } else {
                  let (e0, rest) = exprs.split_first().unwrap();
                  let new_kont = Kont::Let(vars, Vec::with_capacity(len),
                                            rest.to_vec(), eb,
                                            env.clone(), kont_addr);
                  let next_kaddr = store.add_to_store(Val::Kont(new_kont), st);
                  State::new(e0.clone(), env, next_kaddr, st.tick(1))
               }
            } else if let Some((func, arglist)) = matches_apply_expr(list) {
               let new_kont = Kont::ApplyList(None, arglist,
                                           env.clone(), kont_addr);
               let next_kaddr = store.add_to_store(Val::Kont(new_kont), st);
               State::new(func, env, next_kaddr, st.tick(1))
            } else if let Some((prim, arg0, args)) = matches_prim_expr(list) {
               let new_kont = Kont::Prim(
                  prim,
                  Vec::with_capacity(args.len() + 1),
                  args,
                  env.clone(),
                  kont_addr,
               );
               let next_kaddr = store.add_to_store(Val::Kont(new_kont), st);
               State::new(arg0, env, next_kaddr, st.tick(1))
            } else if let Some((prim, listexpr)) = matches_apply_prim_expr(list) {
               let new_kont = Kont::ApplyPrim(prim, kont_addr);
               let next_kaddr = store.add_to_store(Val::Kont(new_kont), st);
               State::new(listexpr, env, next_kaddr, st.tick(1))
            } else if let Some(e) = matches_callcc_expr(list) {
               let new_kont = Kont::Callcc(kont_addr);
               let next_kaddr = store.add_to_store(Val::Kont(new_kont), st);
               State::new(e, env, next_kaddr, st.tick(1))
            } else if let Some((var, e)) = matches_setbang_expr(list) {
               let new_kont = Kont::SetBang(var, kont_addr);
               let next_kaddr = store.add_to_store(Val::Kont(new_kont), st);
               State::new(e, env, next_kaddr, st.tick(1))
            } else {
               // application case
               let (func, args) = list.split_first().expect("Given Empty List");
               let new_kont = Kont::App(
                  Vec::with_capacity(list.len()),
                  args.to_vec(),
                  env.clone(),
                  kont_addr,
               );
               let next_kaddr = store.add_to_store(Val::Kont(new_kont), st);
               State::new(func.clone(), env, next_kaddr, st.tick(1))
            }
         }
         SExpr::Atom(ref _atom) => {
            panic!("Was not handled by atomic case??");
         }
      }
   }
}

pub fn evaluate(ctrl: SExpr) -> (Val, State, Store) {
   // initially the store only has the Empty continuation
   let mut inner = std::collections::HashMap::new();
   inner.insert(Addr(0), Val::Kont(Kont::Empty));
   let mut store = Store(inner);

   let mut st0 = inject(ctrl);
   let mut stepped = step(&st0, &mut store);
   while st0 != stepped {
      st0 = stepped;
      stepped = step(&st0, &mut store);
   }
   let final_value = atomic_eval(&stepped.ctrl, &stepped.env, &store);
   (final_value, stepped, store)
}
