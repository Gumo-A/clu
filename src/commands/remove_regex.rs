use crate::io::{stdin_lines, stdout, write_line};
use clap::Parser;
use regex::Regex;
use std::io;

#[derive(Parser)]
pub struct Args {
    /// Regex; every match is removed from each line.
    pattern: String,
}

pub fn run(args: Args) -> io::Result<()> {
    let re = Regex::new(&args.pattern)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;
    let mut out = stdout();
    for line in stdin_lines() {
        write_line(&mut out, &re.replace_all(&line, ""))?;
    }
    Ok(())
}
