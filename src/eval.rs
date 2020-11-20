//! The evaluator of scheme ASTs.

use crate::common::{PRIMS, State, SExpr, Env, Val, Var, Addr, Store, Kont, Time, Prim, Closure};

fn inject(ctrl: SExpr) -> State {
   // Time 0 was for creation of the state, we start on 1.
   // (becuase mt is addr 0, we need to start with 1)
   State::new(ctrl, Env(im::HashMap::new()), Addr(0), Time(1))
}

fn is_atomic(ctrl: &SExpr) -> bool {
   match ctrl {
      SExpr::List(ref list) => {
         matches!(matches_lambda_expr(list), Some(_))
      },
      SExpr::Atom(_) => true
   }
}

fn matches_number(str: &str) -> Option<i64> {
   str.parse::<i64>().ok()
}

fn matches_boolean(str: &str) -> Option<bool> {
   // because we cant parse #t/#f rn, just use true/false.
   if str == "true" { Some(true) }
   else if str == "false" { Some(false) }
   else { None }
}

fn matches_lambda_expr(list: &[SExpr]) -> Option<(Vec<Var>, SExpr)> {
   if list.len() == 3 && (list[0] == SExpr::Atom("lambda".to_string())
                         || list[0] == SExpr::Atom("λ".to_string())) {
      if let SExpr::List(ref args) = list[1] {
         let mut argvec = Vec::with_capacity(args.len());
         for arg_sexpr in args {
            match arg_sexpr {
               SExpr::List(_) => {panic!("Unexpected list in argument position");},
               SExpr::Atom(ref arg) => {argvec.push(Var(arg.clone()));}
            };
         }
         let body = list[2].clone();
         Some((argvec, body))
      } else {
         panic!("We Dont support vararg lambda at this time");
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

fn matches_let_expr(list: &[SExpr]) -> Option<(Var, SExpr, SExpr)> {
   if list.len() == 3 && list[0] == SExpr::Atom("let".to_string()) {
      match list[1] {
         SExpr::List(ref binding) => {
            let x = match &binding[0] {
               SExpr::List(_) => {panic!("Unexpected list in binding-name position");},
               SExpr::Atom(v) => v.clone(),
            };
            Some((Var(x), binding[1].clone(), list[2].clone()))
         },
         SExpr::Atom(ref _bindlist) => {
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
         SExpr::List(_) => { panic!("Unexpected list in prim-name position"); },
         SExpr::Atom(ref name) => name.clone(),
      };
      let (left, right) = list.split_at(3);
      let arg0 = left[2].clone();
      Some((Prim(primname), arg0, right.to_vec()))
   }  else {
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

fn apply_prim(Prim(op): Prim, args: &[Val]) -> SExpr {
   // idfk why this buffoonery is needed.
   let f = *PRIMS.get::<str>(&op).expect("Prim doesnt exist!");
   f(args)
}

fn atomic_eval(ctrl: &SExpr, env: &Env, store: &Store) -> Val {
   match ctrl {
      SExpr::List(ref list) => {
         if let Some((args, body)) = matches_lambda_expr(list) {
            Val::Closure(Closure(args, body, env.clone()))
         } else {
            panic!("Not given an atomic value, given some list.");
         }
      },
      SExpr::Atom(ref atom) => {
         if let Some(n) = matches_number(atom) { Val::Number(n) }
         else if let Some(b) = matches_boolean(atom) { Val::Boolean(b) }
         else {
            store.get(env.get(Var(atom.clone()))
                         .expect("Atom not in env"))
                 .expect("Atom not in store")
         }
      }
   }
}

fn step_atomic(st: &State, store: &mut Store) -> State {
   let State{ctrl, env, kont_addr, ..} = st.clone();
   let val = atomic_eval(&ctrl, &env, &store);
   let kontval = store.get(kont_addr).expect("Dont Got Kont");
   if let Val::Kont(kont) = kontval {
      match kont {
         Kont::Emptyk => st.clone(), // fixpoint!
         Kont::Ifk(et, ef, ifenv, next_kaddr) => {
            if val == Val::Boolean(false) {
               State::new(ef, ifenv, next_kaddr, st.tick(1))
            } else {
               State::new(et, ifenv, next_kaddr, st.tick(1))
            }
         },
         Kont::Letk(x, eb, letenv, next_kaddr) => {
            let vaddr = store.add_to_store(val, st);
            State::new(eb, letenv.insert(x, vaddr), next_kaddr, st.tick(1))
         },
         Kont::Primk(op, mut done, todo, primenv, next_kaddr) => {
            if todo.is_empty() {
               done.push(val);
               let ae = apply_prim(op, &done);
               State::new(ae, primenv, next_kaddr, st.tick(1))
            } else {
               done.push(val);
               let (head, tail) = todo.split_first().unwrap();
               let new_kont = Kont::Primk(op, done, tail.to_vec(), primenv.clone(), next_kaddr);
               let next_kaddr = store.add_to_store(Val::Kont(new_kont), st);
               State::new(head.clone(), primenv, next_kaddr, st.tick(1))
            }
         },
         Kont::Callcck(next_kaddr) => {
            if let Val::Closure(Closure(params, body, cloenv)) = val {
               if params.len() != 1 { panic!("Calcc lambda only takes 1 argument!");}
               State::new(body, cloenv.insert(params[0].clone(), next_kaddr.clone()),
                          next_kaddr, st.tick(1))
            } else {
               panic!("Callcc only works with lambdas right now! TODO: (call/cc k) support!");
            }
         }
         Kont::Appk(mut done, todo, appenv, next_kaddr) => {
            if todo.is_empty() {
               done.push(val);
               if let (Val::Closure(Closure(params, body, cloenv)), args)
                     = done.split_first().expect("Bad App") {
                  if params.len() != args.len() { panic!("arg # mismatch.");}
                  let mut newenv = cloenv.clone();
                  for (i, (param, arg)) in params.iter().zip(args.iter()).enumerate() {
                     // BIG PROBLEM!
                     // 1) These would get allocated the same addess, because
                     //    the timestamp doesnt change between allocations... duh
                     // Maybe dont use `addr = curtime`? Think!
                     // later:
                     // This solution is _not scientifically kosher!_
                     // consult smarter people for advice.
                     let param_addr = store.add_to_store_offset(arg.clone(), st, i as u64);
                     newenv = newenv.insert(param.clone(), param_addr.clone());
                  }
                  State::new(body.clone(), newenv, next_kaddr, st.tick(1 + (params.len() as u64)))
               } else if let (k@Val::Kont(_), args) = done.split_first().expect("Bad CC App") {
                  if args.len() != 1 { panic!("applying a kont only takes 1 argument.");}

                  // replace the current continuation with the stored one.
                  let new_kaddr = store.add_to_store(k.clone(), st);
                  State::new(ctrl, appenv, new_kaddr, st.tick(1))
               } else {
                  panic!("Closure wasnt head of application");
               }
            } else {
               done.push(val);
               let (head, tail) = todo.split_first().unwrap();
               let new_kont = Kont::Appk(done, tail.to_vec(), appenv.clone(), next_kaddr);
               let next_kaddr = store.add_to_store(Val::Kont(new_kont), st);
               State::new(head.clone(), appenv, next_kaddr, st.tick(1))
            }
         },
      }
   } else {
      panic!("kont_addr not a kont addr!");
   }
}

fn step(st: &State, store: &mut Store) -> State {
   let State{ctrl, env, kont_addr, ..} = st.clone();
   if is_atomic(&ctrl) {
      step_atomic(st, store)
   } else {
      match ctrl {
         SExpr::List(ref list) => {
            if let Some((ec,  et, ef)) = matches_if_expr(list) {
               let new_kont = Kont::Ifk(et, ef, env.clone(), kont_addr);
               let next_kaddr = store.add_to_store(Val::Kont(new_kont), st);
               State::new(ec, env, next_kaddr, st.tick(1))
            } else if let Some((x, ex, eb)) = matches_let_expr(list) {
               let new_kont = Kont::Letk(x, eb, env.clone(), kont_addr);
               let next_kaddr = store.add_to_store(Val::Kont(new_kont), st);
               State::new(ex, env, next_kaddr, st.tick(1))
            } else if let Some((primname, arg0, args)) = matches_prim_expr(list) {
               let new_kont = Kont::Primk(primname, Vec::with_capacity(args.len() + 1), args,
                                          env.clone(), kont_addr);
               let next_kaddr = store.add_to_store(Val::Kont(new_kont), st);
               State::new(arg0, env, next_kaddr, st.tick(1))
            } else if let Some(e) = matches_callcc_expr(list) {
               let new_kont = Kont::Callcck(kont_addr);
               let next_kaddr = store.add_to_store(Val::Kont(new_kont), st);
               State::new(e, env, next_kaddr, st.tick(1))
            } else { // application case
               let (func, args) = list.split_first().expect("Given Empty List");
               let new_kont = Kont::Appk(Vec::with_capacity(list.len()), args.to_vec(),
                                         env.clone(), kont_addr);
               let next_kaddr = store.add_to_store(Val::Kont(new_kont), st);
               State::new(func.clone(), env, next_kaddr, st.tick(1))
            }
         },
         SExpr::Atom(ref _atom) => { panic!("Was not handled by atomic case??"); }
      }
   }
}

pub fn evaluate(ctrl: SExpr) -> (Val, State, Store) {
   // initially the store only has the Empty continuation
   let mut inner = std::collections::HashMap::new();
   inner.insert(Addr(0), Val::Kont(Kont::Emptyk));
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
