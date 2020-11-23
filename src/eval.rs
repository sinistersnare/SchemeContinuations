//! The evaluator of scheme ASTs.

use crate::common::{State, SExprState, Addr, Env, Kont,
                  SExpr, Store, Time, Val};
use crate::evaluation::eval::{expr_step};
use crate::evaluation::apply::{val_step};

fn inject(ctrl: SExpr) -> SExprState {
   // Time 0 was for creation of the state, we start on 1.
   // (becuase mt is addr 0, we need to start with 1)
   SExprState::new(ctrl, Env(im::HashMap::new()), Addr(0), Time(1))
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
