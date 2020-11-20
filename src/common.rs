
use im;

use std::fmt;
use std::collections::HashMap;

#[derive(Hash, Clone, PartialEq, Eq)]
pub enum SExpr {
   List(Vec<SExpr>),
   Atom(String),
}

impl fmt::Debug for SExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    	match self {
    		SExpr::List(ref list) => {
    			write!(f, "(")?;
    			for (i, e) in list.iter().enumerate() {
    				write!(f, "{:?}", e)?;
    				if i+1 != list.len() {
    					write!(f, " ")?;
    				}
    			}
    			write!(f, ")")
    		},
    		SExpr::Atom(ref atom) => {
    			write!(f, "{}", atom)
    		}
    	}
    }
}

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct State {
	pub ctrl: SExpr,
	pub env: Env,
	pub kont_addr: Addr,
	pub time: Time,
}

impl State {
	pub fn new(ctrl: SExpr, env: Env, kont_addr: Addr, time: Time) -> State {
		State { ctrl, env, kont_addr, time }
	}

	pub fn alloc(&self, offset: u64) -> Addr {
		let State{time: Time(t), ..} = self;
		Addr(*t + offset)
	}

	/// Need to give an amount cause multiple allocations
	/// can happen in a single frame (e.g. function application)
	pub fn tick(&self, amt: u64) -> Time {
		let State{time: Time(t), ..} = self;
		Time(t+amt)
	}
}

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct Env(pub im::HashMap<Var, Addr>);
#[derive(Debug, Clone)]
pub struct Store(pub std::collections::HashMap<Addr, Val>);

impl Env {
	pub fn insert(&self, k: Var, v: Addr) -> Env {
		let mut newenv = self.0.clone();
		newenv.insert(k, v);
		Env(newenv)
	}

	pub fn get(&self, var: Var) -> Option<Addr> {
		self.0.get(&var).cloned()
	}
}

impl Store {
	pub fn add_to_store(&mut self, v: Val, st: &State) -> Addr {
		self.add_to_store_offset(v, st, 0)
	}

	pub fn add_to_store_offset(&mut self, v: Val, st: &State, offset: u64) -> Addr {
		let addr = st.alloc(offset);
		self.0.insert(addr.clone(), v);
		addr
	}

	pub fn get(&self, addr: Addr) -> Option<Val> {
		self.0.get(&addr).cloned()
	}
}

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub enum Val {
	Closure(Closure),
	Number(i64),
	Kont(Kont),
	Boolean(bool),
}

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct Closure(pub Vec<Var>, pub SExpr, pub Env);

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub enum Kont {
	Emptyk,
	Ifk(SExpr, SExpr, Env, Addr),
	Letk(Var, SExpr, Env, Addr),
	Primk(Prim, Vec<Val>, Vec<SExpr>, Env, Addr),
	Callcck(Addr),
	Appk(Vec<Val>, Vec<SExpr>, Env, Addr),
}

#[derive(Debug, Clone, Hash,PartialEq, Eq)]
pub struct Var(pub String);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Addr(pub u64);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Time(pub u64);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Prim(pub String);


fn prim_add(args: &[Val]) -> SExpr {
	let mut ret = 0;
	for arg in args {
		if let Val::Number(n) = arg {
			ret += n;
		} else {
			panic!("Not given a number to add!");
		}
	}
	return SExpr::Atom(format!("{}", ret));
}

lazy_static! {
	pub static ref PRIMS: HashMap<&'static str, fn(&[Val]) -> SExpr> = {
		let mut m = HashMap::new();
		m.insert("+", prim_add as fn(&[Val]) -> SExpr);
		m
	};
}
