use crate::io::{stdin_lines, stdout, write_line};
use clap::Parser;
use similar::TextDiff;
use std::io;

#[derive(Parser)]
pub struct Args {
    /// Separator splitting each line into the two strings to compare.
    #[arg(default_value = " ")]
    sep: String,
}

pub fn run(args: Args) -> io::Result<()> {
    let mut out = stdout();
    for line in stdin_lines() {
        let parts: Vec<&str> = line.trim().splitn(2, &args.sep).collect();
        if parts.len() != 2 {
            continue;
        }
        let a = parts[0].trim();
        let b = parts[1].trim();
        let ratio = TextDiff::from_chars(a, b).ratio();
        write_line(&mut out, &format!("{ratio}"))?;
        write_line(&mut out, &format!("\t -> {a}"))?;
        write_line(&mut out, &format!("\t -> {b}"))?;
        write_line(&mut out, "")?;
    }
    Ok(())
}
