use crate::io::{stdin_lines, stdout, write_line};
use clap::Parser;
use rand::RngExt;
use rand::seq::SliceRandom;
use std::io;

#[derive(Parser)]
pub struct Args {
    /// Buffer size: number of lines held in the shuffle reservoir.
    n: usize,
}

pub fn run(args: Args) -> io::Result<()> {
    let n = args.n.max(1);
    let mut rng = rand::rng();
    let mut buffer: Vec<String> = Vec::with_capacity(n);
    let mut out = stdout();

    for line in stdin_lines() {
        let s = line.trim().to_string();
        if buffer.len() < n {
            buffer.push(s);
        } else {
            let idx = rng.random_range(0..buffer.len());
            let old = std::mem::replace(&mut buffer[idx], s);
            write_line(&mut out, &old)?;
        }
    }
    buffer.shuffle(&mut rng);
    for s in &buffer {
        write_line(&mut out, s)?;
    }
    Ok(())
}
