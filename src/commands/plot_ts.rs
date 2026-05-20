use super::plot_distr::run_gnuplot;
use clap::Parser;
use std::io::{self, BufRead};

#[derive(Parser)]
pub struct Args {
    /// Plot title.
    #[arg(long)]
    title: Option<String>,
    /// X-axis label.
    #[arg(long)]
    xlabel: Option<String>,
    /// Y-axis label.
    #[arg(long)]
    ylabel: Option<String>,
}

pub fn run(args: Args) -> io::Result<()> {
    let stdin = io::stdin();
    let mut xs: Vec<String> = Vec::new();
    let mut ys: Vec<f64> = Vec::new();
    for line in stdin.lock().lines() {
        let line = line?;
        let s = line.trim();
        if s.is_empty() {
            continue;
        }
        let Some((x, y_raw)) = s.split_once(char::is_whitespace) else {
            continue;
        };
        let Ok(y) = y_raw.trim().parse::<f64>() else {
            continue;
        };
        xs.push(x.to_string());
        ys.push(y);
    }
    if xs.is_empty() {
        eprintln!("No valid data found.");
        return Ok(());
    }

    let numeric: Option<Vec<f64>> = xs.iter().map(|x| x.parse::<f64>().ok()).collect();

    let mut data = String::new();
    match &numeric {
        Some(nums) => {
            for (x, y) in nums.iter().zip(&ys) {
                data.push_str(&format!("{x} {y}\n"));
            }
        }
        None => {
            for (i, y) in ys.iter().enumerate() {
                data.push_str(&format!("{i} \"{}\" {y}\n", xs[i].replace('"', "'")));
            }
        }
    }

    let mut script = String::new();
    if let Some(t) = &args.title {
        script.push_str(&format!("set title {t:?}\n"));
    }
    if let Some(x) = &args.xlabel {
        script.push_str(&format!("set xlabel {x:?}\n"));
    }
    if let Some(y) = &args.ylabel {
        script.push_str(&format!("set ylabel {y:?}\n"));
    }
    script.push_str("set grid\n");

    match &numeric {
        Some(_) => {
            script.push_str("plot '-' using 1:2 with linespoints pt 7 ps 0.5 notitle\n");
            script.push_str(&data);
            script.push_str("e\n");
        }
        None => {
            let step = (xs.len() / 20).max(1);
            let mut xtics = String::from("set xtics rotate by -45 (");
            let mut first = true;
            for i in (0..xs.len()).step_by(step) {
                if !first {
                    xtics.push_str(", ");
                }
                first = false;
                xtics.push_str(&format!("\"{}\" {}", xs[i].replace('"', "'"), i));
            }
            xtics.push_str(")\n");
            script.push_str(&xtics);
            script.push_str("plot '-' using 1:3 with linespoints pt 7 ps 0.5 notitle\n");
            script.push_str(&data);
            script.push_str("e\n");
        }
    }
    script.push_str("pause -1 'Press Enter to close...'\n");

    run_gnuplot(&script)
}
