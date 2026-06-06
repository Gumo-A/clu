mod commands;
mod io;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;

#[derive(Parser)]
#[command(
    name = "clu",
    about = "A grab bag of stdin-oriented text utilities.",
    version
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Extract the first {...} JSON-looking blob from each line.
    CleanJson,
    /// Split each stdin line on a separator and print non-empty parts on their own lines.
    ExplodeLine(commands::explode_line::Args),
    /// Print the N-th line from stdin (1-indexed).
    GetLine(commands::get_line::Args),
    /// Buffer N lines and join them into one line, separated by SEP.
    JoinLines(commands::join_lines::Args),
    /// Estimate the number of lines in a file from a sample of its bytes.
    LineCountEst(commands::line_count_est::Args),
    /// For each line "A SEP B", print the similarity ratio between A and B.
    LineSim(commands::line_sim::Args),
    /// Reservoir-style shuffle: hold N lines, emit at random as new lines arrive.
    MixLines(commands::mix_lines::Args),
    /// Plot a distribution of numbers from stdin (uses gnuplot).
    PlotDistr(commands::plot_distr::Args),
    /// Print a random float in [0, 1) for each line of stdin.
    RandomNum(commands::random_num::Args),
    /// Replace all matches of REGEX with REPLACEMENT in each line.
    ReplaceRegex(commands::replace_regex::Args),
    /// Python-style slice [A:B:S] applied to each line.
    StrSlice(commands::str_slice::Args),
    /// Print a shell completion script to stdout (bash, zsh, fish, powershell, elvish).
    Completions {
        /// Shell to generate completions for.
        shell: Shell,
    },
}

fn main() {
    io::reset_sigpipe();
    let cli = Cli::parse();
    let result = match cli.cmd {
        Cmd::CleanJson => commands::clean_json::run(),
        Cmd::ExplodeLine(a) => commands::explode_line::run(a),
        Cmd::GetLine(a) => commands::get_line::run(a),
        Cmd::JoinLines(a) => commands::join_lines::run(a),
        Cmd::LineCountEst(a) => commands::line_count_est::run(a),
        Cmd::LineSim(a) => commands::line_sim::run(a),
        Cmd::MixLines(a) => commands::mix_lines::run(a),
        Cmd::PlotDistr(a) => commands::plot_distr::run(a),
        Cmd::RandomNum(a) => commands::random_num::run(a),
        Cmd::ReplaceRegex(a) => commands::replace_regex::run(a),
        Cmd::StrSlice(a) => commands::str_slice::run(a),
        Cmd::Completions { shell } => commands::completions::run(shell, &mut Cli::command()),
    };
    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
