use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub enum RispExp {
    Symbol(String),
    Number(f64),
    List(Vec<RispExp>),
}

#[derive(Debug, PartialEq)]
pub enum RispErr {
    Reason(String),
}

#[derive(Clone)]
pub struct RispEnv {
    pub data: HashMap<String, RispExp>,
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
            (_, rest) = rest.split_first().expect("failed to pop first element");
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

type RispFunc = fn(&[RispExp]) -> Result<RispExp, RispErr>;

pub fn risp_add(args: &[RispExp]) -> Result<RispExp, RispErr> {
    Ok(RispExp::Number(args.iter().map(|x| {
        match x {
            RispExp::Number(v) => v,
            _ => panic!("not a number"),
        }
    }).sum::<f64>()))
}

pub fn standard_env() -> HashMap<String, RispFunc> {
    let mut env = HashMap::new();
    env.insert("+".to_string(), risp_add as RispFunc);
    env
}

pub fn eval(x: RispExp, env: HashMap<String, RispFunc>) -> RispExp {
    RispExp::Symbol("X".to_string())
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
        let output = eval(parse(expr).expect("failed to parse"), standard_env());

        assert_eq!(1, 1);
    }
}
