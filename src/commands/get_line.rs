use crate::io::{stdin_lines, stdout, write_line};
use clap::Parser;
use std::io;

#[derive(Parser)]
pub struct Args {
    /// 1-indexed line number to print.
    n: usize,
}

pub fn run(args: Args) -> io::Result<()> {
    let mut out = stdout();
    for (idx, line) in stdin_lines().enumerate() {
        if idx + 1 == args.n {
            write_line(&mut out, &line)?;
            break;
        }
    }
    Ok(())
}
