use crate::io::{stdin_lines, stdout, write_line};
use std::collections::HashMap;
use std::io;

pub fn run() -> io::Result<()> {
    let mut counts: HashMap<String, u64> = HashMap::new();
    for line in stdin_lines() {
        *counts.entry(line.trim().to_string()).or_insert(0) += 1;
    }
    let mut items: Vec<_> = counts.into_iter().collect();
    items.sort_by(|a, b| a.1.cmp(&b.1));
    let mut out = stdout();
    for (k, v) in &items {
        write_line(&mut out, &format!("{v} {k}"))?;
    }
    Ok(())
}
