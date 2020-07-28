//! The evaluator of scheme ASTs.

use std::collections::HashMap;

use im;

use crate::parse::Expr;


#[derive(Debug)]
pub struct Evaluator {
	// things like GC, symbol table, etc. here.
	symbols: HashMap<String, ScmValue>,
}

impl Evaluator {
	pub fn new() -> Evaluator {
		Evaluator { symbols: HashMap::new(),}
	}

	pub fn eval(&mut self, expr: Expr) -> () {
        println!("Evaling a thing!: {:?}", expr);
        self.eval_inner(expr, im::HashMap::new());
	}

	pub fn eval_inner(&mut self, expr: Expr, locals: im::HashMap<String, ScmValue>) {
        match expr {
        	Expr::Symbol(s) => {
        		// look up symbol in locals then symbol table.
        	},
        	Expr::Numeric(n) => {

        	},
        	Expr::List(xs) => {},
        	Expr::ImproperList(xs) => {},
        	Expr::Null => {},
        }
	}
}
