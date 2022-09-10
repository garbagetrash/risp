use std::collections::HashMap;
use crate::{RispErr, RispExp, RispFunc};

#[derive(Clone)]
pub struct RispEnv {
    pub data: HashMap<String, RispExp>,
    pub funcs: HashMap<String, RispFunc>,
}

impl RispEnv {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            funcs: HashMap::new(),
        }
    }

    pub fn define_procedure(&mut self, symbol: &str, proc: RispFunc) {
        self.data.insert(symbol.to_string(), RispExp::Symbol(symbol.to_string()));
        self.funcs.insert(symbol.to_string(), proc);
    }

    pub fn define_variable(&mut self, symbol: &str, var: &RispExp) {
        self.data.insert(symbol.to_string(), var.clone());
    }
}

impl Default for RispEnv {
    fn default() -> Self {
        Self::new()
    }
}

pub fn risp_add(args: &[RispExp]) -> Result<RispExp, RispErr> {
    Ok(RispExp::Number(args.iter().map(|x| {
        match x {
            RispExp::Number(v) => v,
            _ => panic!("not a number"),
        }
    }).sum::<f64>()))
}

pub fn standard_env() -> RispEnv {
    let mut env = RispEnv::default();
    env.define_procedure("+", risp_add as RispFunc);
    env
}
