# cl-utils

A grab bag of stdin-oriented text utilities, shipped as a single binary: `clu`.

## Install (requires [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html))

```sh
git clone https://github.com/Gumo-A/clu.git
cd clu
cargo install --path .
```

The binary is named `clu` (see `[[bin]]` in `Cargo.toml`).

## Usage

```sh
clu <subcommand> [args...]
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
| `random-num` | Print a random float in `[0, 1)` for each line of stdin. |
| `replace-regex` | Replace all matches of a regex with a replacement in each line. |
| `str-slice` | Python-style slice `[A:B:S]` applied to each line. |
| `completions` | Print a shell completion script (bash, zsh, fish, powershell, elvish). |

Run `clu <subcommand> --help` for per-command options.

## Shell completions

Add one line to your shell's rc file — completions are then generated at shell startup, always matching the installed binary:

```sh
# ~/.bashrc
eval "$(clu completions bash)"

# ~/.zshrc
eval "$(clu completions zsh)"

# ~/.config/fish/config.fish
clu completions fish | source
```

Or, to write a static file instead:

```sh
clu completions bash > /etc/bash_completion.d/clu
clu completions zsh  > "${fpath[1]}/_clu"
```

## Examples

```sh
# Extract JSON blobs from a messy log and count distinct payloads
cat app.log | clu clean-json | clu value-counts

# Quick histogram of response times
awk '{print $7}' access.log | clu plot-distr

# Estimate how many lines are in a huge file without reading it all
clu line-count-est huge.csv
```
