use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub enum RispExp {
    Symbol(String),
    Number(f64),
    List(Vec<RispExp>),
    Procedure(String), // use the string to do a function map lookup
}

#[derive(Debug, PartialEq)]
pub enum RispErr {
    Reason(String),
}

type RispFunc = fn(&[RispExp]) -> Result<RispExp, RispErr>;

#[derive(Clone)]
pub struct RispEnv {
    pub data: HashMap<String, RispExp>,
    pub funcs: HashMap<String, RispFunc>,
}

pub fn tokenize(expr: &str) -> Vec<String> {
    expr.replace("(", " ( ")
        .replace(")", " ) ")
        .split_whitespace()
        .map(|x| x.to_string())
        .collect()
}

pub fn parse(program: &str) -> Result<RispExp, RispErr> {
    read_from_tokens(&tokenize(program))
}

pub fn token_count(re: &RispExp) -> usize {
    match re {
        RispExp::List(v) => 2 + v.iter().map(|x| token_count(x)).sum::<usize>(),
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
            return Ok(RispExp::List(list));
        },
        ")" => Err(RispErr::Reason("unexpected `)`".to_string())),
        _ => Ok(parse_atom(&token)),
    }
}

pub fn parse_atom(token: &str) -> RispExp {
    let potential_float = token.parse();
    match potential_float {
        Ok(v) => RispExp::Number(v),
        Err(_) => RispExp::Symbol(token.to_string().clone()),
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
    let mut datamap = HashMap::new();
    let mut funcmap = HashMap::new();
    let add_symbol = String::from("+");
    datamap.insert(add_symbol.clone(), RispExp::Procedure(add_symbol.clone()));
    funcmap.insert(add_symbol.clone(), risp_add as RispFunc);
    RispEnv { data: datamap, funcs: funcmap }
}

pub fn eval(x: RispExp, env: &mut RispEnv) -> Result<RispExp, RispErr> {
    match x {
        RispExp::Symbol(s) => {
            // Variable lookup
            if let Some(exp) = env.data.get(s.as_str()) {
                return Ok(exp.clone());
            } else {
                return Err(RispErr::Reason("symbol not found".to_string()));
            }
        },
        RispExp::Number(n) => {
            // Numbers are already evaluated as far as we wish them to be
            return Ok(x);
        },
        RispExp::List(v) => {
            // Lists are special. Procedure calls, defines, flow control
            let (first, rest) = v[..].split_first().expect("failed to split list");
            match first {
                RispExp::Procedure(p) => {
                    // Handle procedures
                    let f = env.funcs.get(p).expect("failed to find function");
                    f(rest)
                },
                _ => {
                    return Err(RispErr::Reason(format!("{:?} not implemented", first)));
                },
            }
        },
        RispExp::Procedure(p) => {
            // Does this happen?
            println!("Got naked procedure");
            return Err(RispErr::Reason("not implemented".to_string()));
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let expr = "(+ 10 5)";
        assert_eq!(tokenize(expr), vec!["(", "+", "10", "5", ")"]);
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

    #[test]
    fn test_add() {
        let expr = "(+ 10 5)";
        let mut env = standard_env();
        let output = eval(parse(expr).expect("failed to parse"), &mut env).expect("failed to eval");
        println!("output {:?}", output);

        assert_eq!(output, RispExp::Number(15_f64));
    }
}
