use crate::io::stdin_lines;
use std::io;

pub fn run() -> io::Result<()> {
    let mut hits: u64 = 0;
    let mut misses: u64 = 0;
    for (idx, line) in stdin_lines().enumerate() {
        if idx > 0 && idx % 1024 == 0 {
            println!("HITS {hits} MISSES {misses}");
        }
        match serde_json::from_str::<serde_json::Value>(&line) {
            Ok(_) => hits += 1,
            Err(e) => {
                println!("{e}");
                println!("Failed parse of: {line}");
                misses += 1;
            }
        }
    }
    println!("HITS {hits} MISSES {misses}");
    Ok(())
}
