use std::io::Write;
use std::process::{Command, Stdio};

fn run(args: &[&str], stdin: &str) -> (String, String, i32) {
    let bin = env!("CARGO_BIN_EXE_clu");
    let mut child = Command::new(bin)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn clu");
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(stdin.as_bytes())
        .unwrap();
    let out = child.wait_with_output().expect("wait clu");
    (
        String::from_utf8(out.stdout).unwrap(),
        String::from_utf8(out.stderr).unwrap(),
        out.status.code().unwrap_or(-1),
    )
}

fn stdout_of(args: &[&str], stdin: &str) -> String {
    let (out, err, code) = run(args, stdin);
    assert_eq!(code, 0, "exit={code} stderr={err}");
    out
}

#[test]
fn clean_json_extracts_first_brace_blob() {
    let input = "prefix {\"a\":1} suffix\nno json here\n{\"b\":2}\n";
    let out = stdout_of(&["clean-json"], input);
    assert_eq!(out, "{\"a\":1}\n{\"b\":2}\n");
}

#[test]
fn explode_line_default_space_separator() {
    let out = stdout_of(&["explode-line"], "  a  b   c\nd e\n");
    assert_eq!(out, "a\nb\nc\nd\ne\n");
}

#[test]
fn explode_line_custom_separator() {
    let out = stdout_of(&["explode-line", ","], "a,,b,c\n,x,\n");
    assert_eq!(out, "a\nb\nc\nx\n");
}

#[test]
fn get_line_prints_nth() {
    let input = "one\ntwo\nthree\nfour\n";
    assert_eq!(stdout_of(&["get-line", "1"], input), "one\n");
    assert_eq!(stdout_of(&["get-line", "3"], input), "three\n");
}

#[test]
fn get_line_out_of_range_prints_nothing() {
    assert_eq!(stdout_of(&["get-line", "99"], "a\nb\n"), "");
}

#[test]
fn join_lines_default_pairs_with_space() {
    let out = stdout_of(&["join-lines"], "a\nb\nc\nd\n");
    assert_eq!(out, "a b\nc d\n");
}

#[test]
fn join_lines_flushes_partial_group() {
    let out = stdout_of(&["join-lines", "-n", "3", "-s", ","], "a\nb\nc\nd\ne\n");
    assert_eq!(out, "a,b,c\nd,e\n");
}

#[test]
fn replace_regex_substitutes_matches() {
    let out = stdout_of(
        &["replace-regex", r"\d+", "N"],
        "abc123def\n42\nno digits\n",
    );
    assert_eq!(out, "abcNdef\nN\nno digits\n");
}

#[test]
fn replace_regex_supports_capture_refs() {
    let out = stdout_of(&["replace-regex", r"(\w+)@(\w+)", "$2.$1"], "user@host\n");
    assert_eq!(out, "host.user\n");
}

#[test]
fn replace_regex_invalid_pattern_errors() {
    let (_out, _err, code) = run(&["replace-regex", "(", "x"], "x\n");
    assert_ne!(code, 0);
}

#[test]
fn str_slice_positive_range() {
    let out = stdout_of(&["str-slice", "1", "4"], "abcdef\nhello\n");
    assert_eq!(out, "bcd\nell\n");
}

#[test]
fn str_slice_negative_indices() {
    let out = stdout_of(&["str-slice", "--", "-3", "-1"], "abcdef\n");
    assert_eq!(out, "de\n");
}

#[test]
fn str_slice_reverse_step() {
    let out = stdout_of(&["str-slice", "--step=-1"], "abc\n");
    assert_eq!(out, "cba\n");
}

#[test]
fn str_slice_unicode_chars() {
    // 4 chars: é, l, è, v -> slice [1:3] -> "lè"
    let out = stdout_of(&["str-slice", "1", "3"], "élève\n");
    assert_eq!(out, "lè\n");
}

#[test]
fn str_slice_step_zero_returns_empty() {
    let out = stdout_of(&["str-slice", "--step=0"], "abcdef\n");
    assert_eq!(out, "\n");
}

#[test]
fn line_sim_identical_strings_score_one() {
    let out = stdout_of(&["line-sim"], "hello hello\n");
    let first = out.lines().next().unwrap();
    let ratio: f64 = first.parse().unwrap();
    assert!((ratio - 1.0).abs() < 1e-9, "ratio={ratio}");
}

#[test]
fn line_sim_skips_lines_without_separator() {
    let out = stdout_of(&["line-sim", ","], "no-comma-here\n");
    assert_eq!(out, "");
}

#[test]
fn random_num_emits_one_value_per_input_line() {
    let out = stdout_of(&["random-num"], "x\ny\nz\n");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 3);
    for l in lines {
        let v: f64 = l.parse().expect("parse float");
        assert!((0.0..1.0).contains(&v), "v={v} out of range");
    }
}

#[test]
fn mix_lines_preserves_multiset() {
    let input = "1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n";
    let out = stdout_of(&["mix-lines", "3"], input);
    let mut got: Vec<&str> = out.lines().collect();
    let mut want: Vec<&str> = input.lines().collect();
    got.sort();
    want.sort();
    assert_eq!(got, want);
}

#[test]
fn line_count_est_matches_for_uniform_file() {
    let dir = std::env::temp_dir();
    let path = dir.join(format!("clu-lce-{}.txt", std::process::id()));
    let mut f = std::fs::File::create(&path).unwrap();
    for _ in 0..100 {
        writeln!(f, "abcdefghij").unwrap();
    }
    drop(f);
    let out = stdout_of(
        &["line-count-est", path.to_str().unwrap(), "--sample", "10"],
        "",
    );
    let _ = std::fs::remove_file(&path);
    assert_eq!(out.trim(), "100");
}

#[test]
fn line_count_est_empty_file_is_zero() {
    let path = std::env::temp_dir().join(format!("clu-lce-empty-{}.txt", std::process::id()));
    std::fs::File::create(&path).unwrap();
    let out = stdout_of(&["line-count-est", path.to_str().unwrap()], "");
    let _ = std::fs::remove_file(&path);
    assert_eq!(out.trim(), "0");
}

#[test]
fn completions_prints_bash_script() {
    let out = stdout_of(&["completions", "bash"], "");
    assert!(out.contains("clu"), "expected completion to mention clu");
}
