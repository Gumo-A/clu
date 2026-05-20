use crate::io::{stdin_lines, stdout, write_line};
use clap::Parser;
use std::io;

#[derive(Parser)]
pub struct Args {
    /// Separator to split each line on.
    #[arg(default_value = " ")]
    sep: String,
}

pub fn run(args: Args) -> io::Result<()> {
    let mut out = stdout();
    for line in stdin_lines() {
        for part in line.trim().split(&args.sep) {
            let p = part.trim();
            if !p.is_empty() {
                write_line(&mut out, p)?;
            }
        }
    }
    Ok(())
}
