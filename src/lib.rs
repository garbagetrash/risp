use std::fmt;
pub mod env;
pub use env::{RispEnv, RispFunc, standard_env};

#[cfg(feature = "comms-rs")]
pub mod comms;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum RispExp {
    Bool(bool),
    Symbol(String),
    Number(f64),
    List(Vec<RispExp>),
    Lambda((Box<RispExp>, Box<RispExp>)),
}

impl fmt::Display for RispExp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str_rep = match self {
            RispExp::Bool(b) => b.to_string(),
            RispExp::Symbol(s) => s.clone(),
            RispExp::Number(n) => n.to_string(),
            RispExp::List(v) => {
                let xs: Vec<_> = v.iter().map(|x| x.to_string()).collect();
                format!("({})", xs.join(","))
            },
            RispExp::Lambda((params, body)) => {
                format!("{} {}", params, body)
            },
        };

        write!(f, "{}", str_rep)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum RispErr {
    Reason(String),
}

impl fmt::Display for RispErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RispErr::Reason(s) => write!(f, "Error: {}", s),
        }
    }
}

pub fn tokenize(expr: &str) -> Vec<String> {
    expr.replace('(', " ( ")
        .replace(')', " ) ")
        .split_whitespace()
        .map(|x| x.to_string())
        .collect()
}

pub fn parse(program: &str) -> Result<RispExp, RispErr> {
    read_from_tokens(&tokenize(program))
}

pub fn token_count(re: &RispExp) -> usize {
    match re {
        RispExp::List(v) => 2 + v.iter().map(token_count).sum::<usize>(),
        _ => 1,
    }
}

pub fn read_from_tokens(tokens: &[String]) -> Result<RispExp, RispErr> {
    let (token, mut rest) = tokens.split_first().expect("failed to pop from tokens");
    match token.as_str() {
        "(" => {
            let mut list = vec![];
            while rest[0] != ")" {
                let next_token = read_from_tokens(rest).expect("failed to read from tokens");
                let tlen = token_count(&next_token);
                list.push(next_token);
                (_, rest) = rest.split_at(tlen);
            }
            Ok(RispExp::List(list))
        },
        ")" => Err(RispErr::Reason("unexpected `)`".to_string())),
        _ => Ok(parse_atom(token)),
    }
}

pub fn parse_atom(token: &str) -> RispExp {
    match token {
        "true" => RispExp::Bool(true),
        "false" => RispExp::Bool(false),
        _ => {
            let potential_float = token.parse();
            match potential_float {
                Ok(v) => RispExp::Number(v),
                Err(_) => RispExp::Symbol(token.to_string()),
            }
        }
    }
}

pub fn eval(x: RispExp, env: &mut RispEnv) -> Result<RispExp, RispErr> {
    //println!("eval() x: {:?}", x);
    match x {
        RispExp::Bool(_b) => Ok(x.clone()),
        RispExp::Symbol(s) => {
            // Variable lookup
            if let Some(exp) = env.get(s.as_str()) {
                Ok(exp)
            } else {
                Ok(RispExp::Symbol(s))
            }
        },
        RispExp::Number(_n) => {
            // Numbers are already evaluated as far as we wish them to be
            Ok(x)
        },
        RispExp::List(v) => {
            // Lists are special. Procedure calls, defines, flow control
            let (first, rest) = v[..].split_first().expect("failed to split list");
            match first {
                RispExp::Symbol(p) => {
                    // Handle procedures
                    if let Some(f) = env.get_function(p) {
                        f(rest, env)
                    } else {
                        // Handle lambdas
                        if let Some(l) = env.get(p) {
                            match l {
                                RispExp::Lambda((params, body)) => {
                                    let params = if let RispExp::List(pars) = *params {
                                        pars
                                    } else {
                                        return Err(RispErr::Reason("lambda parameters must be a RispExp::List".to_string()));
                                    };

                                    if rest.len() != params.len() {
                                        return Err(RispErr::Reason(
                                            "length of passed args doesn't match expected parameters".to_string()
                                            ));
                                    }

                                    // If we got here it seems things parsed correctly

                                    // Create our inner scope, add parameters to it
                                    let mut inner_scope = RispEnv::new();
                                    for (sym, arg) in params.iter().zip(rest.iter()) {
                                        if let RispExp::Symbol(s) = sym {
                                            inner_scope.define_variable(s, arg)
                                        } else {
                                            return Err(RispErr::Reason("parameter RispExp didn't evaluate to symbol".to_string()));
                                        }
                                    }
                                    inner_scope.outer = Some(env);

                                    eval(*body, &mut inner_scope)
                                },
                                _ => Err(RispErr::Reason(format!("failed to find function or lambda {:?}", first))),
                            }
                        } else {
                            Err(RispErr::Reason(format!("failed to find function or lambda {:?}", first)))
                        }
                    }
                },
                _ => {
                    Err(RispErr::Reason(format!("{:?} not implemented", first)))
                },
            }
        },
        RispExp::Lambda(_) => Err(RispErr::Reason("Unexpected form".to_string())),
    }
}

pub fn eval_to_number(x: &RispExp, env: &mut RispEnv) -> Result<f64, RispErr> {
    match eval(x.clone(), env) {
        Ok(re) => {
            match re {
                RispExp::Number(n) => Ok(n),
                _ => Err(RispErr::Reason(format!("{:?} did not eval to a number", x))),
            }
        },
        Err(rerr) => Err(rerr),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let expr = "(+ 10 5)";
        assert_eq!(tokenize(expr), vec!["(", "+", "10", "5", ")"]);

        let expr = "(begin (define r 10) (* pi (* r r)))";
        assert_eq!(tokenize(expr),
            vec!["(", "begin", "(", "define", "r", "10", ")", "(", "*", "pi",
                 "(", "*", "r", "r", ")", ")", ")"
            ]);
    }

    #[test]
    fn test_parse() {
        let expr = "(+ 10 5)";
        let output = parse(expr).expect("failed to parse");
        let truth = RispExp::List(vec![RispExp::Symbol("+".to_string()), RispExp::Number(10_f64), RispExp::Number(5_f64)]);
        assert_eq!(output, truth);

        let expr = "(begin (define r 10) (* pi (* r r)))";
        let output = parse(expr).expect("failed to parse");
        let truth = RispExp::List(vec![
            RispExp::Symbol("begin".to_string()),
            RispExp::List(vec![
                RispExp::Symbol("define".to_string()),
                RispExp::Symbol("r".to_string()),
                RispExp::Number(10_f64),
            ]),
            RispExp::List(vec![
                RispExp::Symbol("*".to_string()),
                RispExp::Symbol("pi".to_string()),
                RispExp::List(vec![
                    RispExp::Symbol("*".to_string()),
                    RispExp::Symbol("r".to_string()),
                    RispExp::Symbol("r".to_string()),
                ]),
            ]),
        ]);
        assert_eq!(output, truth);
    }
}
