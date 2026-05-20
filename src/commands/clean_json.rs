use crate::io::{stdin_lines, stdout, write_line};
use regex::Regex;
use std::io;

pub fn run() -> io::Result<()> {
    let re = Regex::new(r"\{.*\}").unwrap();
    let mut out = stdout();
    for line in stdin_lines() {
        if let Some(m) = re.find(&line) {
            write_line(&mut out, m.as_str())?;
        }
    }
    Ok(())
}
