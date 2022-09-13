use std::io;
use std::io::Write;

use risp::*;

#[cfg(feature = "comms-rs")]
use risp::comms::*;

#[cfg(feature = "comms-rs")]
fn repl_env<'a>() -> RispEnv<'a> {
    comms_env()
}

#[cfg(not(feature = "comms-rs"))]
fn repl_env<'a>() -> RispEnv<'a> {
    standard_env()
}

fn main() {
    let mut env = repl_env();

    loop {
        print!("risp > ");
        io::stdout().flush().expect("failed to flush stdout");

        let mut expr_str = String::new();
        let nbytes = io::stdin().read_line(&mut expr_str).expect("failed to read line");

        if nbytes == 0 {
            // EOF on empty line means ctrl + d was hit, so bail
            println!("");
            break;
        }

        // Handle some REPL type things first
        let trimmed = &expr_str.trim_end();
        if "exit".eq_ignore_ascii_case(trimmed) {
            // Bail when user types "exit"
            break;
        } else if "".eq_ignore_ascii_case(trimmed) {
            // Handle empty line by restarting loop
            continue;
        }

        // Now try to treat it as risp code
        let expr = parse(expr_str.as_str()).expect("failed to parse line");

        match eval(expr, &mut env) {
            Ok(re) => println!("{}", re),
            Err(rerr) => println!("{}", rerr),
        }
    }
}
