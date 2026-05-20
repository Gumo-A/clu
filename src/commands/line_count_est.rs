use clap::Parser;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

#[derive(Parser)]
pub struct Args {
    /// File to estimate line count for.
    file: String,
    /// Sample size in lines.
    #[arg(long, default_value_t = 10_000)]
    sample: usize,
}

pub fn run(args: Args) -> io::Result<()> {
    let file_size = std::fs::metadata(&args.file)?.len();
    let f = BufReader::new(File::open(&args.file)?);

    let mut sample_bytes: u64 = 0;
    let mut sample_lines: u64 = 0;
    for line in f.lines().take(args.sample) {
        let line = line?;
        sample_bytes += line.len() as u64 + 1; // +1 for the newline
        sample_lines += 1;
    }
    if sample_lines == 0 {
        println!("0");
        return Ok(());
    }
    let bytes_per_line = sample_bytes as f64 / sample_lines as f64;
    let estimation = (file_size as f64 / bytes_per_line).round() as u64;
    println!("{estimation}");
    Ok(())
}
