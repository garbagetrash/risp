use std::collections::HashMap;
use std::f64;
use std::sync::{Arc, Mutex};
use crate::{eval, eval_to_number, RispErr, RispExp};

pub type RispFunc = fn(&[RispExp], &mut RispEnv) -> Result<RispExp, RispErr>;

#[cfg(feature = "comms-rs")]
use comms_rs::node::graph::Graph;

#[derive(Clone)]
pub struct RispEnv<'a> {
    data: HashMap<String, RispExp>,
    funcs: HashMap<String, RispFunc>,
    pub outer: Option<&'a RispEnv<'a>>,

    #[cfg(feature = "comms-rs")]
    pub comms_graphs: Vec<Arc<Mutex<Graph>>>,
}

impl<'a> RispEnv<'a> {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            funcs: HashMap::new(),
            outer: None,
            #[cfg(feature = "comms-rs")]
            comms_graphs: vec![],
        }
    }

    pub fn define_procedure(&mut self, symbol: &str, proc: RispFunc) {
        self.data.insert(symbol.to_string(), RispExp::Symbol(symbol.to_string()));
        self.funcs.insert(symbol.to_string(), proc);
    }

    pub fn define_variable(&mut self, symbol: &str, var: &RispExp) {
        self.data.insert(symbol.to_string(), var.clone());
    }

    pub fn get(&self, symbol: &str) -> Option<RispExp> {
        if let Some(s) = self.data.get(symbol) {
            Some(s.clone())
        } else if let Some(outer) = &self.outer {
            outer.get(symbol)
        } else {
            None
        }
    }

    pub fn get_function(&self, symbol: &str) -> Option<RispFunc> {
        if let Some(s) = self.funcs.get(symbol) {
            Some(*s)
        } else if let Some(outer) = &self.outer {
            outer.get_function(symbol)
        } else {
            None
        }
    }
}

impl<'a> Default for RispEnv<'a> {
    fn default() -> Self {
        Self::new()
    }
}

pub fn risp_if(args: &[RispExp], env: &mut RispEnv) -> Result<RispExp, RispErr> {
    let (predicate, alternatives) = args.split_first().expect("`if` requires at least 3 arguments");
    let predicate = eval(predicate.clone(), env).expect("failed to evaluate predicate");
    match predicate {
        RispExp::Bool(truth) => {
            if truth {
                // true
                eval(alternatives[0].clone(), env)
            } else {
                // false
                eval(alternatives[1].clone(), env)
            }

        },
        _ => Err(RispErr::Reason(format!("{:?} does not evaluate to a boolean", predicate))),
    }
}

pub fn risp_let(args: &[RispExp], env: &mut RispEnv) -> Result<RispExp, RispErr> {
    let (symbol, expr) = args.split_first().expect("`let` requires at least 3 arguments");
    // The fact that we don't eval(symbol) means the first argument has to be
    // the symbol alone, no fanciness with lists allowed.
    match symbol {
        RispExp::Symbol(s) => {
            // NOTE: we're tossing out expr[1..] here, intentionally.
            let expr = eval(expr[0].clone(), env)?;
            env.data.insert(s.clone(), expr.clone());
            Ok(expr)
        },
        _ => Err(RispErr::Reason(format!("{:?} does not evaluate to a symbol", symbol))),
    }
}

pub fn risp_lambda(args: &[RispExp], _env: &mut RispEnv) -> Result<RispExp, RispErr> {
    let (params, func) = args.split_first().expect("`fn` requires 2 arguments");

    if func.len() > 1 {
        return Err(RispErr::Reason("`fn` definition expected to only have 2 arguments".to_string()));
    }

    Ok(RispExp::Lambda((Box::new(params.clone()), Box::new(func[0].clone()))))
}

pub fn risp_add(args: &[RispExp], env: &mut RispEnv) -> Result<RispExp, RispErr> {
    let mut total = 0.0;
    for arg in args {
        if let Ok(n) = eval_to_number(arg, env) {
            total += n;
        } else {
            return Err(RispErr::Reason(format!("{:?} not a number", arg)));
        };
    }
    Ok(RispExp::Number(total))
}

pub fn risp_subtract(args: &[RispExp], env: &mut RispEnv) -> Result<RispExp, RispErr> {
    //println!("- args: {:?}", args);
    let (first, rest_nums) = args.split_first().expect("`-` requires at least 2 arguments");
    let num1 = if let Ok(n) = eval_to_number(first, env) {
        n
    } else {
        return Err(RispErr::Reason(format!("{:?} not a number", first)));
    };

    let mut sum_right = 0.0;
    for num in rest_nums {
        let num = if let Ok(n) = eval_to_number(num, env) {
            n
        } else {
            return Err(RispErr::Reason(format!("{:?} not a number", first)));
        };

        sum_right += num;
    }

    Ok(RispExp::Number(num1 - sum_right))
}

pub fn risp_multiply(args: &[RispExp], env: &mut RispEnv) -> Result<RispExp, RispErr> {
    let mut total = 1.0;
    for arg in args {
        if let Ok(n) = eval_to_number(arg, env) {
            total *= n;
        } else {
            return Err(RispErr::Reason(format!("{:?} not a number", arg)));
        };
    }
    Ok(RispExp::Number(total))
}

pub fn risp_divide(args: &[RispExp], env: &mut RispEnv) -> Result<RispExp, RispErr> {
    let (first, rest_nums) = args.split_first().expect("`/` requires at least 2 arguments");
    if rest_nums.len() > 1 {
        return Err(RispErr::Reason("`/` takes exactly 2 arguments".to_string()));
    }

    let numerator = if let Ok(n) = eval_to_number(first, env) {
        n
    } else {
        return Err(RispErr::Reason(format!("{:?} not a number", first)));
    };

    let denominator = if let Ok(n) = eval_to_number(&rest_nums[0], env) {
        n
    } else {
        return Err(RispErr::Reason(format!("{:?} not a number", first)));
    };

    Ok(RispExp::Number(numerator / denominator))
}

pub fn risp_cosine(args: &[RispExp], env: &mut RispEnv) -> Result<RispExp, RispErr> {
    if args.len() > 1 {
        return Err(RispErr::Reason("`cos` takes exactly 1 argument".to_string()));
    }
    let num = if let Ok(n) = eval_to_number(&args[0], env) {
        n
    } else {
        return Err(RispErr::Reason(format!("{:?} not a number", args[0])));
    };
    Ok(RispExp::Number(num.cos()))
}

pub fn risp_sine(args: &[RispExp], env: &mut RispEnv) -> Result<RispExp, RispErr> {
    if args.len() > 1 {
        return Err(RispErr::Reason("`sin` takes exactly 1 argument".to_string()));
    }
    let num = if let Ok(n) = eval_to_number(&args[0], env) {
        n
    } else {
        return Err(RispErr::Reason(format!("{:?} not a number", args[0])));
    };
    Ok(RispExp::Number(num.sin()))
}

pub fn risp_tangent(args: &[RispExp], env: &mut RispEnv) -> Result<RispExp, RispErr> {
    if args.len() > 1 {
        return Err(RispErr::Reason("`tan` takes exactly 1 argument".to_string()));
    }
    let num = if let Ok(n) = eval_to_number(&args[0], env) {
        n
    } else {
        return Err(RispErr::Reason(format!("{:?} not a number", args[0])));
    };
    Ok(RispExp::Number(num.tan()))
}

pub fn risp_acos(args: &[RispExp], env: &mut RispEnv) -> Result<RispExp, RispErr> {
    if args.len() > 1 {
        return Err(RispErr::Reason("`acos` takes exactly 1 argument".to_string()));
    }
    let num = if let Ok(n) = eval_to_number(&args[0], env) {
        n
    } else {
        return Err(RispErr::Reason(format!("{:?} not a number", args[0])));
    };
    Ok(RispExp::Number(num.acos()))
}

pub fn risp_asin(args: &[RispExp], env: &mut RispEnv) -> Result<RispExp, RispErr> {
    if args.len() > 1 {
        return Err(RispErr::Reason("`asin` takes exactly 1 argument".to_string()));
    }
    let num = if let Ok(n) = eval_to_number(&args[0], env) {
        n
    } else {
        return Err(RispErr::Reason(format!("{:?} not a number", args[0])));
    };
    Ok(RispExp::Number(num.asin()))
}

pub fn risp_atan(args: &[RispExp], env: &mut RispEnv) -> Result<RispExp, RispErr> {
    if args.len() > 1 {
        return Err(RispErr::Reason("`atan` takes exactly 1 argument".to_string()));
    }
    let num = if let Ok(n) = eval_to_number(&args[0], env) {
        n
    } else {
        return Err(RispErr::Reason(format!("{:?} not a number", args[0])));
    };
    Ok(RispExp::Number(num.atan()))
}

pub fn risp_log(args: &[RispExp], env: &mut RispEnv) -> Result<RispExp, RispErr> {
    if args.len() > 1 {
        return Err(RispErr::Reason("`log` takes exactly 1 argument".to_string()));
    }
    let num = if let Ok(n) = eval_to_number(&args[0], env) {
        n
    } else {
        return Err(RispErr::Reason(format!("{:?} not a number", args[0])));
    };
    Ok(RispExp::Number(num.ln()))
}

pub fn risp_log2(args: &[RispExp], env: &mut RispEnv) -> Result<RispExp, RispErr> {
    if args.len() > 1 {
        return Err(RispErr::Reason("`log2` takes exactly 1 argument".to_string()));
    }
    let num = if let Ok(n) = eval_to_number(&args[0], env) {
        n
    } else {
        return Err(RispErr::Reason(format!("{:?} not a number", args[0])));
    };
    Ok(RispExp::Number(num.log2()))
}

pub fn risp_log10(args: &[RispExp], env: &mut RispEnv) -> Result<RispExp, RispErr> {
    if args.len() > 1 {
        return Err(RispErr::Reason("`log10` takes exactly 1 argument".to_string()));
    }
    let num = if let Ok(n) = eval_to_number(&args[0], env) {
        n
    } else {
        return Err(RispErr::Reason(format!("{:?} not a number", args[0])));
    };
    Ok(RispExp::Number(num.log10()))
}

pub fn risp_sqrt(args: &[RispExp], env: &mut RispEnv) -> Result<RispExp, RispErr> {
    if args.len() > 1 {
        return Err(RispErr::Reason("`sqrt` takes exactly 1 argument".to_string()));
    }
    let num = if let Ok(n) = eval_to_number(&args[0], env) {
        n
    } else {
        return Err(RispErr::Reason(format!("{:?} not a number", args[0])));
    };
    Ok(RispExp::Number(num.sqrt()))
}

pub fn risp_exp(args: &[RispExp], env: &mut RispEnv) -> Result<RispExp, RispErr> {
    if args.len() > 1 {
        return Err(RispErr::Reason("`exp` takes exactly 1 argument".to_string()));
    }
    let num = if let Ok(n) = eval_to_number(&args[0], env) {
        n
    } else {
        return Err(RispErr::Reason(format!("{:?} not a number", args[0])));
    };
    Ok(RispExp::Number(num.exp()))
}

pub fn risp_abs(args: &[RispExp], env: &mut RispEnv) -> Result<RispExp, RispErr> {
    if args.len() > 1 {
        return Err(RispErr::Reason("`abs` takes exactly 1 argument".to_string()));
    }
    let num = if let Ok(n) = eval_to_number(&args[0], env) {
        n
    } else {
        return Err(RispErr::Reason(format!("{:?} not a number", args[0])));
    };
    Ok(RispExp::Number(num.abs()))
}

pub fn risp_pow(args: &[RispExp], env: &mut RispEnv) -> Result<RispExp, RispErr> {
    if args.len() != 2 {
        return Err(RispErr::Reason("`pow` takes exactly 2 arguments".to_string()));
    }

    let base = if let Ok(n) = eval_to_number(&args[0], env) {
        n
    } else {
        return Err(RispErr::Reason(format!("{:?} not a number", args[0])));
    };

    let power = if let Ok(n) = eval_to_number(&args[1], env) {
        n
    } else {
        return Err(RispErr::Reason(format!("{:?} not a number", args[1])));
    };

    Ok(RispExp::Number(base.powf(power)))
}

pub fn risp_eq(args: &[RispExp], _env: &mut RispEnv) -> Result<RispExp, RispErr> {
    let (left, others) = args.split_first().expect("`=` requires at least 2 arguments");

    for other in others {
        if left != other {
            return Ok(RispExp::Bool(false));
        }
    }

    Ok(RispExp::Bool(true))
}

pub fn risp_neq(args: &[RispExp], _env: &mut RispEnv) -> Result<RispExp, RispErr> {
    let (left, others) = args.split_first().expect("`!=` requires at least 2 arguments");

    for other in others {
        if left != other {
            return Ok(RispExp::Bool(true));
        }
    }

    Ok(RispExp::Bool(false))
}

pub fn risp_gt(args: &[RispExp], env: &mut RispEnv) -> Result<RispExp, RispErr> {
    let (left, others) = args.split_first().expect("`>` requires at least 2 arguments");

    let left = eval_to_number(left, env)?;
    for other in others {
        let other = eval_to_number(other, env)?;
        if left <= other {
            return Ok(RispExp::Bool(false));
        }
    }

    Ok(RispExp::Bool(true))
}

pub fn risp_gte(args: &[RispExp], env: &mut RispEnv) -> Result<RispExp, RispErr> {
    let (left, others) = args.split_first().expect("`>=` requires at least 2 arguments");

    let left = eval_to_number(left, env)?;
    for other in others {
        let other = eval_to_number(other, env)?;
        if left < other {
            return Ok(RispExp::Bool(false));
        }
    }

    Ok(RispExp::Bool(true))
}

pub fn risp_lt(args: &[RispExp], _env: &mut RispEnv) -> Result<RispExp, RispErr> {
    let (left, others) = args.split_first().expect("`<` requires at least 2 arguments");

    for other in others {
        if left >= other {
            return Ok(RispExp::Bool(false));
        }
    }

    Ok(RispExp::Bool(true))
}

pub fn risp_lte(args: &[RispExp], _env: &mut RispEnv) -> Result<RispExp, RispErr> {
    let (left, others) = args.split_first().expect("`<=` requires at least 2 arguments");

    for other in others {
        if left > other {
            return Ok(RispExp::Bool(false));
        }
    }

    Ok(RispExp::Bool(true))
}

pub fn standard_env<'a>() -> RispEnv<'a> {
    let mut env = RispEnv::default();
    env.define_variable("pi", &RispExp::Number(f64::consts::PI));
    env.define_procedure("if", risp_if as RispFunc);
    env.define_procedure("let", risp_let as RispFunc);
    env.define_procedure("fn", risp_lambda as RispFunc);
    env.define_procedure("+", risp_add as RispFunc);
    env.define_procedure("-", risp_subtract as RispFunc);
    env.define_procedure("*", risp_multiply as RispFunc);
    env.define_procedure("/", risp_divide as RispFunc);
    env.define_procedure("cos", risp_cosine as RispFunc);
    env.define_procedure("sin", risp_sine as RispFunc);
    env.define_procedure("tan", risp_tangent as RispFunc);
    env.define_procedure("acos", risp_acos as RispFunc);
    env.define_procedure("asin", risp_asin as RispFunc);
    env.define_procedure("atan", risp_atan as RispFunc);
    env.define_procedure("log", risp_log as RispFunc);
    env.define_procedure("log2", risp_log2 as RispFunc);
    env.define_procedure("log10", risp_log10 as RispFunc);
    env.define_procedure("sqrt", risp_sqrt as RispFunc);
    env.define_procedure("exp", risp_exp as RispFunc);
    env.define_procedure("abs", risp_abs as RispFunc);
    env.define_procedure("pow", risp_pow as RispFunc);
    env.define_procedure("=", risp_eq as RispFunc);
    env.define_procedure("!=", risp_neq as RispFunc);
    env.define_procedure(">", risp_gt as RispFunc);
    env.define_procedure(">=", risp_gte as RispFunc);
    env.define_procedure("<", risp_lt as RispFunc);
    env.define_procedure("<=", risp_lte as RispFunc);
    env
}

#[cfg(test)]
mod tests {
    use crate::*;
    use std::f64;

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
    fn test_subtract() {
        let expr = "(- 10 5)";
        let mut env = standard_env();
        let output = eval(parse(expr).expect("failed to parse"), &mut env).expect("failed to eval");
        assert_eq!(output, RispExp::Number(5_f64));

        let expr = "(- 10 (- 8 3) 3 1 -12)";
        let mut env = standard_env();
        let output = eval(parse(expr).expect("failed to parse"), &mut env).expect("failed to eval");
        assert_eq!(output, RispExp::Number(13_f64));
    }

    #[test]
    fn test_multiply() {
        let expr = "(* 10 5)";
        let mut env = standard_env();
        let output = eval(parse(expr).expect("failed to parse"), &mut env).expect("failed to eval");
        assert_eq!(output, RispExp::Number(50_f64));

        let expr = "(* 10 (- 8 3) 3 1)";
        let mut env = standard_env();
        let output = eval(parse(expr).expect("failed to parse"), &mut env).expect("failed to eval");
        assert_eq!(output, RispExp::Number(150_f64));
    }

    #[test]
    fn test_divide() {
        let expr = "(/ 10 5)";
        let mut env = standard_env();
        let output = eval(parse(expr).expect("failed to parse"), &mut env).expect("failed to eval");
        assert_eq!(output, RispExp::Number(2_f64));

        let expr = "(/ 150 (- 8 3))";
        let mut env = standard_env();
        let output = eval(parse(expr).expect("failed to parse"), &mut env).expect("failed to eval");
        assert_eq!(output, RispExp::Number(30_f64));
    }

    #[test]
    fn test_trigonometry() {
        let expr = "(cos 0)";
        let mut env = standard_env();
        let output = eval(parse(expr).expect("failed to parse"), &mut env).expect("failed to eval");
        assert_eq!(output, RispExp::Number(1_f64));

        let expr = "(cos pi)";
        let output = eval(parse(expr).expect("failed to parse"), &mut env).expect("failed to eval");
        assert_eq!(output, RispExp::Number(-1_f64));

        let expr = "(sin 0)";
        let output = eval(parse(expr).expect("failed to parse"), &mut env).expect("failed to eval");
        assert_eq!(output, RispExp::Number(0_f64));

        let expr = "(sin (/ pi 2))";
        let output = eval(parse(expr).expect("failed to parse"), &mut env).expect("failed to eval");
        assert_eq!(output, RispExp::Number(1_f64));

        let expr = "(tan 0)";
        let output = eval(parse(expr).expect("failed to parse"), &mut env).expect("failed to eval");
        assert_eq!(output, RispExp::Number(0_f64));

        let expr = "(tan (/ pi 4))";
        let output = eval(parse(expr).expect("failed to parse"), &mut env).expect("failed to eval");
        assert_eq!(output, RispExp::Number((f64::consts::PI / 4.0).tan()));

        let expr = "(tan (atan (/ pi 4)))";
        let output = eval(parse(expr).expect("failed to parse"), &mut env).expect("failed to eval");
        assert_eq!(output, RispExp::Number(f64::consts::PI / 4.0));

        let expr = "(cos (acos (/ pi 4)))";
        let output = eval(parse(expr).expect("failed to parse"), &mut env).expect("failed to eval");
        assert_eq!(output, RispExp::Number(f64::consts::PI / 4.0));

        let expr = "(sin (asin (/ pi 4)))";
        let output = eval(parse(expr).expect("failed to parse"), &mut env).expect("failed to eval");
        assert_eq!(output, RispExp::Number(f64::consts::PI / 4.0));
    }

    #[test]
    fn test_bool() {
        let expr = "(> 10 5 4 2 1 9)";
        let mut env = standard_env();
        let output = eval(parse(expr).expect("failed to parse"), &mut env).expect("failed to eval");
        assert_eq!(output, RispExp::Bool(true));

        let expr = "(> 10 5 4 2 11 9)";
        let mut env = standard_env();
        let output = eval(parse(expr).expect("failed to parse"), &mut env).expect("failed to eval");
        assert_eq!(output, RispExp::Bool(false));

        let expr = "(> 10 5 4 2 10 9)";
        let mut env = standard_env();
        let output = eval(parse(expr).expect("failed to parse"), &mut env).expect("failed to eval");
        assert_eq!(output, RispExp::Bool(false));

        let expr = "(= 10 10 10 10)";
        let mut env = standard_env();
        let output = eval(parse(expr).expect("failed to parse"), &mut env).expect("failed to eval");
        assert_eq!(output, RispExp::Bool(true));

        let expr = "(<= 3 5 7 5 3 5)";
        let mut env = standard_env();
        let output = eval(parse(expr).expect("failed to parse"), &mut env).expect("failed to eval");
        assert_eq!(output, RispExp::Bool(true));

        let expr = "(< 3 5 7 5 3 5)";
        let mut env = standard_env();
        let output = eval(parse(expr).expect("failed to parse"), &mut env).expect("failed to eval");
        assert_eq!(output, RispExp::Bool(false));

        let expr = "(!= 10 10 10 9 10)";
        let mut env = standard_env();
        let output = eval(parse(expr).expect("failed to parse"), &mut env).expect("failed to eval");
        assert_eq!(output, RispExp::Bool(true));

        let expr = "(!= 10 10 10 (+ 3 (+ 3 3)) 10)";
        let mut env = standard_env();
        let output = eval(parse(expr).expect("failed to parse"), &mut env).expect("failed to eval");
        assert_eq!(output, RispExp::Bool(true));

        let expr = "(!= 10 10 10 10)";
        let mut env = standard_env();
        let output = eval(parse(expr).expect("failed to parse"), &mut env).expect("failed to eval");
        assert_eq!(output, RispExp::Bool(false));
    }

    #[test]
    fn test_if() {
        let expr = "(if (!= 10 10 10 10) asdf 1)";
        let mut env = standard_env();
        let output = eval(parse(expr).expect("failed to parse"), &mut env).expect("failed to eval");
        assert_eq!(output, RispExp::Number(1.0));

        let expr = "(if (= 10 10 10) asdf 1)";
        let mut env = standard_env();
        let output = eval(parse(expr).expect("failed to parse"), &mut env).expect("failed to eval");
        assert_eq!(output, RispExp::Symbol("asdf".to_string()));

        let expr = "(if (< 10 11 9) asdf 1)";
        let mut env = standard_env();
        let output = eval(parse(expr).expect("failed to parse"), &mut env).expect("failed to eval");
        assert_eq!(output, RispExp::Number(1.0));

        let expr = "(if (< 10 11 9) asdf (+ 1 (- 3 2)))";
        let mut env = standard_env();
        let output = eval(parse(expr).expect("failed to parse"), &mut env).expect("failed to eval");
        assert_eq!(output, RispExp::Number(2.0));
    }

    #[test]
    fn test_let() {
        let expr = "(let a 3)";
        let mut env = standard_env();
        let output = eval(parse(expr).expect("failed to parse"), &mut env).expect("failed to eval");
        assert_eq!(output, RispExp::Number(3.0));

        let expr = "(let b 5)";
        let output = eval(parse(expr).expect("failed to parse"), &mut env).expect("failed to eval");
        assert_eq!(output, RispExp::Number(5.0));

        let expr = "(- b a)";
        let output = eval(parse(expr).expect("failed to parse"), &mut env).expect("failed to eval");
        assert_eq!(output, RispExp::Number(2.0));

        let expr = "(if (= a b) (let a 5) (let a 42))";
        let output = eval(parse(expr).expect("failed to parse"), &mut env).expect("failed to eval");
        assert_eq!(output, RispExp::Number(42.0));

        let expr = "a";
        let output = eval(parse(expr).expect("failed to parse"), &mut env).expect("failed to eval");
        assert_eq!(output, RispExp::Number(42.0));
    }

    #[test]
    fn test_outer_env() {
        let mut env = standard_env();
        let expr = "(let b 5)";
        let output = eval(parse(expr).expect("failed to parse"), &mut env).expect("failed to eval");
        assert_eq!(output, RispExp::Number(5.0));

        let mut inner_env = RispEnv::new();
        inner_env.outer = Some(&env);
        let expr = "(let a 3)";
        let output = eval(parse(expr).expect("failed to parse"), &mut inner_env).expect("failed to eval");
        assert_eq!(output, RispExp::Number(3.0));

        let expr = "a";
        let output = eval(parse(expr).expect("failed to parse"), &mut inner_env).expect("failed to eval");
        assert_eq!(output, RispExp::Number(3.0));

        let expr = "b";
        let output = eval(parse(expr).expect("failed to parse"), &mut inner_env).expect("failed to eval");
        assert_eq!(output, RispExp::Number(5.0));
    }

    #[test]
    fn test_lambda() {
        let mut env = standard_env();
        let expr = "(let addone (fn (x) (+ x 1)))";
        eval(parse(expr).expect("failed to parse"), &mut env).expect("failed to eval");

        let expr = "(addone 4.3)";
        let output = eval(parse(expr).expect("failed to parse"), &mut env).expect("failed to eval");
        assert_eq!(output, RispExp::Number(5.3));
    }
}
