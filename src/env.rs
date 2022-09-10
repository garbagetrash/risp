use std::collections::HashMap;
use crate::{eval, RispErr, RispExp};

type RispFunc = fn(&[RispExp], &mut RispEnv) -> Result<RispExp, RispErr>;

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

pub fn risp_add(args: &[RispExp], env: &mut RispEnv) -> Result<RispExp, RispErr> {
    Ok(RispExp::Number(args.iter().map(|x| {
        *match x {
            RispExp::Number(v) => v,
            RispExp::List(_) => {
                let expr = eval(x.clone(), env).expect("failed to eval list");
                if let RispExp::Number(n) = expr {
                    return n;
                } else {
                    panic!("not a number");
                }
            },
            _ => panic!("not a number"),
        }
    }).sum::<f64>()))
}

pub fn risp_sub(args: &[RispExp], env: &mut RispEnv) -> Result<RispExp, RispErr> {
    let (first, rest_nums) = args.split_first().expect("`-` requires at least 2 arguments");
    let num1 = match first {
        RispExp::Number(v) => *v,
        RispExp::List(_) => {
            if let Ok(expr) = eval(first.clone(), env) {
                match expr {
                    RispExp::Number(n) => n,
                    _ => panic!("not a number"),
                }
            } else {
                panic!("failed to eval first");
            }
        },
        _ => panic!("not a number"),
    };

    let num2 = rest_nums.iter().map(|x| {
        *match x {
            RispExp::Number(v) => v,
            RispExp::List(_) => {
                let expr = eval(x.clone(), env).expect("failed to eval list");
                if let RispExp::Number(n) = expr {
                    return n;
                } else {
                    panic!("not a number");
                }
            },
            _ => panic!("not a number"),
        }
    }).sum::<f64>();
    Ok(RispExp::Number(num1 - num2))
}

pub fn standard_env() -> RispEnv {
    let mut env = RispEnv::default();
    env.define_procedure("+", risp_add as RispFunc);
    env.define_procedure("-", risp_sub as RispFunc);
    env
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_add() {
        let expr = "(+ 10 5)";
        let mut env = standard_env();
        let output = eval(parse(expr).expect("failed to parse"), &mut env).expect("failed to eval");
        assert_eq!(output, RispExp::Number(15_f64));

        let expr = "(+ 10 5 3 1 -12)";
        let mut env = standard_env();
        let output = eval(parse(expr).expect("failed to parse"), &mut env).expect("failed to eval");
        assert_eq!(output, RispExp::Number(7_f64));

        let expr = "(+ 10 (+ 5 (+ 1 2)) 1 -12)";
        let mut env = standard_env();
        let output = eval(parse(expr).expect("failed to parse"), &mut env).expect("failed to eval");
        assert_eq!(output, RispExp::Number(7_f64));
    }

    #[test]
    fn test_sub() {
        let expr = "(- 10 5)";
        let mut env = standard_env();
        let output = eval(parse(expr).expect("failed to parse"), &mut env).expect("failed to eval");
        assert_eq!(output, RispExp::Number(5_f64));

        let expr = "(- 10 (- 8 3) 3 1 -12)";
        let mut env = standard_env();
        let output = eval(parse(expr).expect("failed to parse"), &mut env).expect("failed to eval");
        assert_eq!(output, RispExp::Number(13_f64));
    }
}
