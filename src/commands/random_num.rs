use crate::io::{stdin_lines, stdout, write_line};
use rand::RngExt;
use std::io;

pub fn run() -> io::Result<()> {
    let mut rng = rand::rng();
    let mut out = stdout();
    for _ in stdin_lines() {
        let v: f64 = rng.random();
        write_line(&mut out, &format!("{v}"))?;
    }
    Ok(())
}
