use std::io::{self, BufRead, BufWriter, StdoutLock, Write};

pub fn reset_sigpipe() {
    #[cfg(unix)]
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
}

pub fn stdin_lines() -> impl Iterator<Item = String> {
    io::stdin().lock().lines().map_while(Result::ok)
}

pub fn stdout() -> BufWriter<StdoutLock<'static>> {
    BufWriter::new(io::stdout().lock())
}

pub fn write_line(out: &mut impl Write, s: &str) -> io::Result<()> {
    out.write_all(s.as_bytes())?;
    out.write_all(b"\n")
}
