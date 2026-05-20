use clap::Parser;
use std::io::{self, BufRead, Write};
use std::process::{Command, Stdio};

#[derive(Parser)]
pub struct Args {
    /// Apply natural log transform to each value.
    #[arg(long)]
    log: bool,
    /// Number of histogram bins.
    #[arg(long, default_value_t = 50)]
    bins: usize,
}

pub fn run(args: Args) -> io::Result<()> {
    let stdin = io::stdin();
    let mut values: Vec<f64> = Vec::new();
    for line in stdin.lock().lines() {
        let line = line?;
        let s = line.trim();
        let Ok(mut v) = s.parse::<f64>() else { continue };
        if args.log {
            v = v.ln();
        }
        if v.is_finite() {
            values.push(v);
        }
    }
    if values.is_empty() {
        eprintln!("No valid numeric data found.");
        return Ok(());
    }

    let (counts, lo, hi) = histogram(&values, args.bins);
    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;
    let var = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
    let std = var.sqrt();
    let (min, max) = values
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &v| {
            (lo.min(v), hi.max(v))
        });

    let width = (hi - lo) / args.bins as f64;
    let mut data = String::new();
    for (i, c) in counts.iter().enumerate() {
        let center = lo + (i as f64 + 0.5) * width;
        data.push_str(&format!("{center} {c}\n"));
    }

    let title = format!(
        "Histogram (N={}, mean={:.3}, std={:.3}, min={:.3}, max={:.3})",
        values.len(),
        mean,
        std,
        min,
        max
    );

    let script = format!(
        "set title {title:?}\n\
         set style fill solid 0.6 border -1\n\
         set boxwidth {width}\n\
         set xlabel 'Value'\n\
         set ylabel 'Count'\n\
         plot '-' using 1:2 with boxes notitle\n\
         {data}e\n\
         pause -1 'Press Enter to close...'\n"
    );
    run_gnuplot(&script)
}

fn histogram(values: &[f64], bins: usize) -> (Vec<u64>, f64, f64) {
    let (mut lo, mut hi) = values
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &v| {
            (lo.min(v), hi.max(v))
        });
    if lo == hi {
        lo -= 0.5;
        hi += 0.5;
    }
    let span = hi - lo;
    let mut counts = vec![0u64; bins];
    for &v in values {
        let mut idx = ((v - lo) / span * bins as f64) as isize;
        if idx < 0 {
            idx = 0;
        }
        if idx as usize >= bins {
            idx = bins as isize - 1;
        }
        counts[idx as usize] += 1;
    }
    (counts, lo, hi)
}

pub(crate) fn run_gnuplot(script: &str) -> io::Result<()> {
    let mut child = Command::new("gnuplot")
        .arg("-persist")
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| {
            io::Error::new(
                e.kind(),
                format!("failed to launch gnuplot (is it installed?): {e}"),
            )
        })?;
    child
        .stdin
        .as_mut()
        .expect("piped stdin")
        .write_all(script.as_bytes())?;
    child.wait()?;
    Ok(())
}
