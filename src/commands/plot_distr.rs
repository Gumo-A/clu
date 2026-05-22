use clap::Parser;
use gnuplot::{AutoOption, AxesCommon, Figure, PlotOption, RGBString};
use std::io::{self, BufRead};

const DEFAULT_SAMPLE: usize = 100_000;
const MIN_BINS: usize = 4;
const MAX_BINS: usize = 128;
const HARD_BIN_CAP: usize = 10_000;

#[derive(Parser)]
pub struct Args {
    /// Apply natural log transform to each value.
    #[arg(long)]
    log: bool,
    /// Number of leading values used to estimate histogram bin layout.
    #[arg(long, default_value_t = DEFAULT_SAMPLE)]
    sample: usize,
}

pub fn run(args: Args) -> io::Result<()> {
    let sample_size = args.sample.max(2);
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    let mut lineno: usize = 0;

    let mut stats = Stats::new();
    let mut sample: Vec<f64> = Vec::with_capacity(sample_size);

    // Phase 1: buffer the first `sample_size` valid values so we can derive a
    // bin layout from the whole sample. If the stream ends first, we use
    // everything we got.
    while sample.len() < sample_size {
        let Some(line) = lines.next() else { break };
        lineno += 1;
        let Some(v) = parse_value(&line?, lineno, args.log) else {
            continue;
        };
        stats.push(v);
        sample.push(v);
    }

    if stats.n == 0 {
        eprintln!("No valid numeric data found.");
        return Ok(());
    }

    let mut hist = Histogram::from_sample(&sample);

    // Phase 2: stream the remaining values straight into the histogram and
    // stats accumulator without retaining them. If the stream ended during
    // Phase 1 we still have the full sequence, so we can also render the
    // series line plot in a multiplot view.
    let mut streamed = false;
    for line in lines {
        streamed = true;
        lineno += 1;
        let Some(v) = parse_value(&line?, lineno, args.log) else {
            continue;
        };
        stats.push(v);
        hist.push(v);
    }

    if streamed {
        plot_hist(&hist, &stats)
    } else {
        multiplot_distr(&hist, &stats, &sample)
    }
}

fn parse_value(line: &str, lineno: usize, log: bool) -> Option<f64> {
    let s = line.trim();
    let Ok(mut v) = s.parse::<f64>() else {
        eprintln!("warning: line {lineno}: could not parse {s:?} as a number, skipping");
        return None;
    };
    if log {
        v = v.ln();
    }
    if !v.is_finite() {
        eprintln!("warning: line {lineno}: value {s:?} is not finite after transform, skipping");
        return None;
    }
    Some(v)
}

/// Online (single-pass) accumulator for mean, variance, min, and max.
///
/// Uses Welford's algorithm for numerical stability; never stores the data.
#[derive(Debug)]
struct Stats {
    n: u64,
    mean: f64,
    m2: f64,
    min: f64,
    max: f64,
}

impl Stats {
    fn new() -> Self {
        Self {
            n: 0,
            mean: 0.0,
            m2: 0.0,
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
        }
    }

    fn push(&mut self, x: f64) {
        self.n += 1;
        let delta = x - self.mean;
        self.mean += delta / self.n as f64;
        let delta2 = x - self.mean;
        self.m2 += delta * delta2;
        if x < self.min {
            self.min = x;
        }
        if x > self.max {
            self.max = x;
        }
    }

    fn std(&self) -> f64 {
        if self.n < 2 {
            0.0
        } else {
            (self.m2 / self.n as f64).sqrt()
        }
    }
}

/// Fixed-width histogram that grows its bin array on either side when a value
/// falls outside the current range. The bin *width* is locked at construction
/// time so counts always represent equal-sized buckets.
struct Histogram {
    lo: f64,
    width: f64,
    bins: Vec<u64>,
}

impl Histogram {
    /// Builds an empty histogram with bin layout derived from a sample, then
    /// folds the sample's values back into the resulting bins.
    fn from_sample(sample: &[f64]) -> Self {
        let (lo, width, n_bins) = estimate_layout(sample);
        let mut h = Self {
            lo,
            width,
            bins: vec![0; n_bins],
        };
        for &v in sample {
            h.push(v);
        }
        h
    }

    fn hi(&self) -> f64 {
        self.lo + self.width * self.bins.len() as f64
    }

    fn push(&mut self, x: f64) {
        let idx_f = (x - self.lo) / self.width;
        if idx_f >= 0.0 && (idx_f as usize) < self.bins.len() {
            self.bins[idx_f as usize] += 1;
            return;
        }

        if idx_f < 0.0 {
            let needed = ((-idx_f).ceil() as usize).max(1);
            if self.bins.len() + needed > HARD_BIN_CAP {
                self.bins[0] += 1;
                return;
            }
            let mut new_bins = vec![0u64; needed];
            new_bins.extend_from_slice(&self.bins);
            self.bins = new_bins;
            self.lo -= self.width * needed as f64;
            self.bins[0] += 1;
        } else {
            let target = idx_f as usize;
            let needed = target + 1 - self.bins.len();
            if self.bins.len() + needed > HARD_BIN_CAP {
                let last = self.bins.len() - 1;
                self.bins[last] += 1;
                return;
            }
            self.bins.resize(self.bins.len() + needed, 0);
            self.bins[target] += 1;
        }
    }

    fn centers(&self) -> Vec<f64> {
        (0..self.bins.len())
            .map(|i| self.lo + (i as f64 + 0.5) * self.width)
            .collect()
    }
}

/// Picks a histogram `(lo, width, n_bins)` from a sample.
///
/// Uses Scott's rule (`w = 3.5 * std / n^(1/3)`) for bin width, then clamps the
/// resulting bin count to `[MIN_BINS, MAX_BINS]` and rounds the width so the
/// sample range divides evenly. Falls back to a fixed split when std is zero.
fn estimate_layout(sample: &[f64]) -> (f64, f64, usize) {
    let n = sample.len() as f64;
    let mean = sample.iter().sum::<f64>() / n;
    let var = sample.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
    let std = var.sqrt();

    let (mut lo, mut hi) = sample
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), &v| {
            (a.min(v), b.max(v))
        });
    if hi <= lo {
        lo -= 0.5;
        hi += 0.5;
    }

    let scott = if std > 0.0 {
        3.5 * std / n.cbrt()
    } else {
        (hi - lo) / MIN_BINS as f64
    };
    let raw_bins = ((hi - lo) / scott).ceil() as usize;
    let n_bins = raw_bins.clamp(MIN_BINS, MAX_BINS);
    let width = (hi - lo) / n_bins as f64;
    (lo, width, n_bins)
}

fn plot_hist(hist: &Histogram, stats: &Stats) -> io::Result<()> {
    let centers = hist.centers();
    let title = format!(
        "Histogram (N={}, mean={:.3}, std={:.3}, min={:.3}, max={:.3})",
        stats.n,
        stats.mean,
        stats.std(),
        stats.min,
        stats.max,
    );

    let mut fig = Figure::new();
    fig.axes2d()
        .set_title(&title, &[])
        .set_x_label("Value", &[])
        .set_y_label("Count", &[])
        .boxes(
            &centers,
            &hist.bins,
            &[
                PlotOption::FillAlpha(0.6),
                PlotOption::Color(RGBString("blue")),
                PlotOption::BorderColor(RGBString("black")),
            ],
        )
        .set_x_range(AutoOption::Fix(hist.lo), AutoOption::Fix(hist.hi()));

    let mut handle = fig.show().map_err(|e| {
        io::Error::other(format!("failed to launch gnuplot (is it installed?): {e}"))
    })?;
    handle
        .wait()
        .map_err(|e| io::Error::other(format!("gnuplot exited with error: {e}")))?;
    Ok(())
}

/// Renders a 2x2 multiplot: histogram (top-left), box plot (top-right), and
/// a line plot of the values in arrival order spanning the full bottom row.
fn multiplot_distr(hist: &Histogram, stats: &Stats, data: &[f64]) -> io::Result<()> {
    let centers = hist.centers();
    let title = format!(
        "Distribution (N={}, mean={:.3}, std={:.3}, min={:.3}, max={:.3})",
        stats.n,
        stats.mean,
        stats.std(),
        stats.min,
        stats.max,
    );

    let (q1, median, q3, whisker_lo, whisker_hi) = box_plot_stats(data);

    let mut fig = Figure::new();
    fig.set_multiplot_layout(2, 2).set_title(&title);

    // Top-left: histogram.
    fig.axes2d()
        .set_pos(0.0, 0.5)
        .set_size(0.5, 0.5)
        .set_title("Histogram", &[])
        .set_x_label("Value", &[])
        .set_y_label("Count", &[])
        .boxes(
            &centers,
            &hist.bins,
            &[
                PlotOption::FillAlpha(0.6),
                PlotOption::Color(RGBString("blue")),
                PlotOption::BorderColor(RGBString("black")),
            ],
        )
        .set_x_range(AutoOption::Fix(hist.lo), AutoOption::Fix(hist.hi()));

    // Top-right: box plot.
    fig.axes2d()
        .set_pos(0.5, 0.5)
        .set_size(0.5, 0.5)
        .set_title("Box plot", &[])
        .set_y_label("Value", &[])
        .set_x_range(AutoOption::Fix(-1.0), AutoOption::Fix(1.0))
        .set_x_ticks(None, &[], &[])
        .box_and_whisker(
            &[0.0f64],
            &[q1],
            &[whisker_lo],
            &[whisker_hi],
            &[q3],
            &[
                PlotOption::Color(RGBString("blue")),
                PlotOption::FillAlpha(0.6),
                PlotOption::BorderColor(RGBString("black")),
                PlotOption::BoxWidth(vec![0.5]),
            ],
        )
        .points(
            &[0.0f64],
            &[median],
            &[
                PlotOption::Color(RGBString("red")),
                PlotOption::PointSymbol('O'),
                PlotOption::PointSize(0.9),
                PlotOption::PointSize(3.0),
            ],
        )
        .set_y_range(
            AutoOption::Fix(whisker_lo - (whisker_hi * 0.1)),
            AutoOption::Fix(whisker_hi + (whisker_hi * 0.1)),
        );

    // Bottom row spanning full width: series in arrival order.
    let xs: Vec<usize> = (0..data.len()).collect();
    fig.axes2d()
        .set_pos(0.0, 0.0)
        .set_size(1.0, 0.5)
        .set_title("Series", &[])
        .set_x_label("Index", &[])
        .set_y_label("Value", &[])
        .lines(&xs, data, &[PlotOption::Color(RGBString("blue"))]);

    let mut handle = fig.show().map_err(|e| {
        io::Error::other(format!("failed to launch gnuplot (is it installed?): {e}"))
    })?;
    handle
        .wait()
        .map_err(|e| io::Error::other(format!("gnuplot exited with error: {e}")))?;
    Ok(())
}

/// Returns `(q1, median, q3, whisker_lo, whisker_hi)` for a Tukey box plot.
///
/// Whiskers are clamped to the most extreme data point still inside
/// `[Q1 - 1.5*IQR, Q3 + 1.5*IQR]`.
fn box_plot_stats(data: &[f64]) -> (f64, f64, f64, f64, f64) {
    let mut sorted: Vec<f64> = data.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let quantile = |p: f64| -> f64 {
        let pos = p * (sorted.len() - 1) as f64;
        let lo = pos.floor() as usize;
        let hi = pos.ceil() as usize;
        let frac = pos - lo as f64;
        sorted[lo] * (1.0 - frac) + sorted[hi] * frac
    };
    let q1 = quantile(0.25);
    let median = quantile(0.5);
    let q3 = quantile(0.75);
    let iqr = q3 - q1;
    let lo_fence = q1 - 1.5 * iqr;
    let hi_fence = q3 + 1.5 * iqr;
    let whisker_lo = sorted
        .iter()
        .copied()
        .find(|&v| v >= lo_fence)
        .unwrap_or(sorted[0]);
    let whisker_hi = sorted
        .iter()
        .rev()
        .copied()
        .find(|&v| v <= hi_fence)
        .unwrap_or(*sorted.last().unwrap());
    (q1, median, q3, whisker_lo, whisker_hi)
}

pub(crate) fn run_gnuplot(script: &str) -> io::Result<()> {
    use std::io::Write;
    use std::process::{Command, Stdio};
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

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f64, b: f64, tol: f64) {
        assert!((a - b).abs() <= tol, "{a} != {b} (tol {tol})");
    }

    #[test]
    fn stats_matches_batch_computation() {
        let xs = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let mut s = Stats::new();
        for x in xs {
            s.push(x);
        }
        let n = xs.len() as f64;
        let mean = xs.iter().sum::<f64>() / n;
        let var = xs.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
        assert_eq!(s.n, xs.len() as u64);
        approx_eq(s.mean, mean, 1e-12);
        approx_eq(s.std(), var.sqrt(), 1e-12);
        approx_eq(s.min, 1.0, 0.0);
        approx_eq(s.max, 10.0, 0.0);
    }

    #[test]
    fn stats_is_numerically_stable_for_shifted_data() {
        // Welford should hold up where the naive sum-of-squares formula loses
        // precision. Shift by a large offset to stress the difference.
        let offset = 1e9;
        let mut s = Stats::new();
        for i in 0..1000 {
            s.push(offset + i as f64);
        }
        approx_eq(s.mean, offset + 499.5, 1e-6);
        // Population variance of 0..1000 = (n^2 - 1)/12 = 83333.25
        approx_eq(s.std(), 83333.25_f64.sqrt(), 1e-6);
    }

    #[test]
    fn histogram_total_count_equals_input_size() {
        let sample: Vec<f64> = (0..200).map(|i| i as f64 / 10.0).collect();
        let mut h = Histogram::from_sample(&sample);
        // Push some extra values, some inside and some outside the sample range.
        let extras = [-5.0, 25.0, 7.5, 0.0, 19.999];
        for &v in &extras {
            h.push(v);
        }
        let total: u64 = h.bins.iter().sum();
        assert_eq!(total as usize, sample.len() + extras.len());
    }

    #[test]
    fn histogram_extends_for_values_above_range() {
        let sample: Vec<f64> = (0..100).map(|i| i as f64).collect();
        let mut h = Histogram::from_sample(&sample);
        let original_hi = h.hi();
        let original_len = h.bins.len();
        h.push(original_hi + 50.0 * h.width); // well past the right edge
        assert!(h.bins.len() > original_len);
        assert!(h.hi() > original_hi);
        // The new value lives in the last (newly added) bin.
        assert_eq!(*h.bins.last().unwrap(), 1);
    }

    #[test]
    fn histogram_extends_for_values_below_range() {
        let sample: Vec<f64> = (0..100).map(|i| i as f64).collect();
        let mut h = Histogram::from_sample(&sample);
        let original_lo = h.lo;
        let original_len = h.bins.len();
        h.push(original_lo - 50.0 * h.width); // well past the left edge
        assert!(h.bins.len() > original_len);
        assert!(h.lo < original_lo);
        // The new value lives in the first (newly prepended) bin.
        assert_eq!(h.bins[0], 1);
    }

    #[test]
    fn histogram_keeps_constant_bin_width_after_extending() {
        let sample: Vec<f64> = (0..100).map(|i| i as f64).collect();
        let mut h = Histogram::from_sample(&sample);
        let width = h.width;
        h.push(-500.0);
        h.push(1000.0);
        approx_eq(h.width, width, 0.0);
    }

    #[test]
    fn estimate_layout_handles_constant_sample() {
        // All values identical — Scott's rule would divide by zero; fallback should engage.
        let sample = vec![3.14; 50];
        let (lo, width, n_bins) = estimate_layout(&sample);
        assert!(width > 0.0);
        assert!(n_bins >= MIN_BINS);
        assert!(lo < 3.14 && lo + width * n_bins as f64 > 3.14);
    }

    #[test]
    fn estimate_layout_clamps_bin_count() {
        let sample: Vec<f64> = (0..500).map(|i| i as f64).collect();
        let (_, _, n_bins) = estimate_layout(&sample);
        assert!((MIN_BINS..=MAX_BINS).contains(&n_bins));
    }
}
