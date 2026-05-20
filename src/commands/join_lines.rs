use crate::io::{stdin_lines, stdout, write_line};
use clap::Parser;
use std::io;

#[derive(Parser)]
pub struct Args {
    /// Number of lines per joined group.
    #[arg(short, default_value_t = 2)]
    n: usize,
    /// Separator between joined lines.
    #[arg(short, default_value = " ")]
    s: String,
}

pub fn run(args: Args) -> io::Result<()> {
    let n = args.n.max(1);
    let mut out = stdout();
    let mut buffer: Vec<String> = Vec::with_capacity(n);
    for line in stdin_lines() {
        buffer.push(line.trim().to_string());
        if buffer.len() == n {
            write_line(&mut out, &buffer.join(&args.s))?;
            buffer.clear();
        }
    }
    if !buffer.is_empty() {
        write_line(&mut out, &buffer.join(&args.s))?;
    }
    Ok(())
}
