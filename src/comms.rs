use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use crate::{eval, eval_to_number, RispErr, RispExp, RispFunc, standard_env, RispEnv};

use comms_rs::prelude::*;
use comms_rs::node::graph::Graph;
use num::{Complex, Num, Zero};
use rand::prelude::*;

#[derive(Node)]
struct QpskMod {
    pub output: NodeSender<Vec<Complex<f64>>>,
    filter_state: Vec<Complex<f64>>,
    rrc_taps: Vec<Complex<f64>>,
}

impl QpskMod {

    pub fn new() -> Self {
        let filter_state = vec![Complex::zero(); 32];
        let sam_per_sym = 2.0;
        let rrc_taps =
            comms_rs::util::math::rrc_taps(32, sam_per_sym, 0.25).expect("failed to create RRC taps");
        Self {
            output: Default::default(),
            filter_state,
            rrc_taps,
        }
    }

    pub fn run(&mut self) -> Result<Vec<Complex<f64>>, NodeError> {
        let dist = rand::distributions::Uniform::new(0u8, 2u8);
        let mut rng = rand::thread_rng();
        let mut bits: Vec<u8> = vec![];
        for _ in 0..4096 {
            bits.push(rng.sample(&dist));
        }
        let qpsk_mod: Vec<Complex<f64>> = bits
            .iter()
            .step_by(2)
            .zip(bits.iter().skip(1).step_by(2))
            .map(|(&x, &y)| {
                std::f64::consts::FRAC_1_SQRT_2 * (2.0 * Complex::new(x as f64, y as f64) - Complex::new(1.0, 1.0))
            })
            .collect();
        let mut upsample = vec![Complex::zero(); 4096 * 2];
        let mut ix = 0;
        for samp in qpsk_mod {
            upsample[ix] = samp;
            ix += 4;
        }
        let data = comms_rs::filter::fir::batch_fir(&upsample, &self.rrc_taps, &mut self.filter_state);
        Ok(data)
    }
}

#[derive(Node)]
struct PrinterNode<T>
where
    T: Num + Copy + Send + Debug,
{
    pub input: NodeReceiver<Vec<T>>,
    pub output: NodeSender<Vec<T>>,
}

impl<T> PrinterNode<T>
where
    T: Num + Copy + Send + Debug,
{
    pub fn new() -> Self {
        Self {
            input: Default::default(),
            output: Default::default(),
        }
    }

    pub fn run(&mut self, input: Vec<T>) -> Result<Vec<T>, NodeError> {
        println!("{:?}", input);
        Ok(input)
    }
}

pub fn comms_qpsk(args: &[RispExp], env: &mut RispEnv) -> Result<RispExp, RispErr> {
    // 1 param: output node
    if args.len() != 1 {
        return Err(RispErr::Reason("`qpsk` expects 1 argument".to_string()));
    }

    let mut graph = env.comms_graphs[0].lock().expect("failed to lock Graph");
    graph.add_node(Arc::new(Mutex::new(QpskMod::new())));

    Err(RispErr::Reason("not implemented".to_string()))
}

pub fn comms_env<'a>() -> RispEnv<'a> {
    let mut env = standard_env();
    env.comms_graphs.push(Arc::new(Mutex::new(Graph::new(None))));
    env.define_procedure("qpsk", comms_qpsk as RispFunc);
    env
}
