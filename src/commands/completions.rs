use clap_complete::Shell;
use std::io;

pub fn run(shell: Shell, cmd: &mut clap::Command) -> io::Result<()> {
    let name = cmd.get_name().to_string();
    clap_complete::generate(shell, cmd, name, &mut io::stdout());
    Ok(())
}
