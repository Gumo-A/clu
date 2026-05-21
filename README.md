# cli-utils

A grab bag of stdin-oriented text utilities, shipped as a single binary: `cliu`.

## Install

```sh
cargo install --path .
```

The binary is named `cliu` (see `[[bin]]` in `Cargo.toml`).

## Usage

```sh
cliu <subcommand> [args...]
```

Most subcommands read lines from stdin and write to stdout, so they compose with pipes.

### Subcommands

| Command | Description |
| --- | --- |
| `clean-json` | Extract the first `{...}` JSON-looking blob from each line. |
| `explode-line` | Split each line on a separator; print non-empty parts on their own lines. |
| `get-line` | Print the N-th line from stdin (1-indexed). |
| `join-lines` | Buffer N lines and join them into one line with a separator. |
| `line-count-est` | Estimate a file's line count from a sample of its bytes. |
| `line-sim` | For each line `A SEP B`, print the similarity ratio between A and B. |
| `mix-lines` | Reservoir-style shuffle: hold N lines, emit at random as new lines arrive. |
| `plot-distr` | Plot a distribution of numbers from stdin (uses gnuplot). |
| `plot-ts` | Plot a time series `x y` per line from stdin (uses gnuplot). |
| `random-num` | Print a random float in `[0, 1)` for each line of stdin. |
| `remove-regex` | Remove all matches of a regex from each line. |
| `str-slice` | Python-style slice `[A:B:S]` applied to each line. |
| `try-parse-json` | Try to parse each line as JSON; report hits, misses, and parse errors. |
| `value-counts` | Count occurrences of each distinct line and print sorted. |
| `completions` | Print a shell completion script (bash, zsh, fish, powershell, elvish). |

Run `cliu <subcommand> --help` for per-command options.

## Shell completions

Add one line to your shell's rc file — completions are then generated at shell startup, always matching the installed binary:

```sh
# ~/.bashrc
eval "$(cliu completions bash)"

# ~/.zshrc
eval "$(cliu completions zsh)"

# ~/.config/fish/config.fish
cliu completions fish | source
```

Or, to write a static file instead:

```sh
cliu completions bash > /etc/bash_completion.d/cliu
cliu completions zsh  > "${fpath[1]}/_cliu"
```

## Examples

```sh
# Extract JSON blobs from a messy log and count distinct payloads
cat app.log | cliu clean-json | cliu value-counts

# Quick histogram of response times
awk '{print $7}' access.log | cliu plot-distr

# Estimate how many lines are in a huge file without reading it all
cliu line-count-est huge.csv
```
