use crate::io::{stdin_lines, stdout, write_line};
use clap::Parser;
use std::io;

#[derive(Parser)]
pub struct Args {
    /// Start index (Python-style; negative counts from the end).
    a: Option<i64>,
    /// End index (exclusive; negative counts from the end).
    b: Option<i64>,
    /// Step.
    #[arg(short, long, default_value_t = 1)]
    step: i64,
}

pub fn run(args: Args) -> io::Result<()> {
    let mut out = stdout();
    for line in stdin_lines() {
        let trimmed = line.trim_end_matches(['\r', '\n']);
        let sliced = python_slice(trimmed, args.a, args.b, args.step);
        write_line(&mut out, &sliced)?;
    }
    Ok(())
}

/// Mirror Python's string slicing: index by Unicode characters, allow negative
/// indices, negative step reverses, missing bounds default per step direction.
fn python_slice(s: &str, a: Option<i64>, b: Option<i64>, step: i64) -> String {
    if step == 0 {
        return String::new();
    }
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len() as i64;

    let resolve = |i: i64| -> i64 {
        if i < 0 { (i + len).max(0) } else { i.min(len) }
    };

    let (start, end) = if step > 0 {
        (a.map(resolve).unwrap_or(0), b.map(resolve).unwrap_or(len))
    } else {
        (
            a.map(|i| if i < 0 { (i + len).max(-1) } else { i.min(len - 1) })
                .unwrap_or(len - 1),
            b.map(|i| if i < 0 { (i + len).max(-1) } else { i.min(len - 1) })
                .unwrap_or(-1),
        )
    };

    let mut out = String::new();
    let mut i = start;
    while (step > 0 && i < end) || (step < 0 && i > end) {
        if (0..len).contains(&i) {
            out.push(chars[i as usize]);
        }
        i += step;
    }
    out
}
