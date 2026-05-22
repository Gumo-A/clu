use crate::io::{stdin_lines, stdout, write_line};
use clap::{Parser, ValueEnum};
use rand;
use rand::RngExt;
use std::io;

#[derive(Parser)]
pub struct Args {
    /// Distribution of the random values
    #[arg(short, long = "distr")]
    distr: Option<Distr>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Distr {
    /// Standard normal
    Norm,
    /// Standard Uniform
    Unif,
}

pub fn run(args: Args) -> io::Result<()> {
    let mut rng = rand::rng();
    let mut out = stdout();
    for _ in stdin_lines() {
        let v: f64 = match args.distr {
            Some(Distr::Norm) => rng.sample(rand_distr::StandardNormal),
            Some(Distr::Unif) => rng.sample(rand_distr::StandardUniform),
            None => rng.sample(rand_distr::StandardUniform),
        };
        write_line(&mut out, &format!("{v}"))?;
    }
    Ok(())
}
