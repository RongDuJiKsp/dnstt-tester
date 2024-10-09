use std::process::Stdio;
use tokio::io;
use tokio::process::{Child, Command};

pub fn load_env_and_run(exe: &str, arg: &str, port: u16) -> io::Result<Child> {
    Command::new(exe)
        .args(
            arg.replace("&[port]", &port.to_string())
                .split(" ")
                .collect::<Vec<_>>(),
        )
        .kill_on_drop(true)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
}
