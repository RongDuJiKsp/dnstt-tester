use std::collections::HashMap;
use std::process::Stdio;
use tokio::io;
use tokio::process::{Child, Command};
pub fn run_exe_with_env(
    exe: &str,
    raw_args: &str,
    env: &HashMap<String, String>,
) -> io::Result<Child> {
    Command::new(exe)
        .args(
            &env.iter()
                .fold(raw_args.to_string(), |s, (from, to)| {
                    s.replace(&format!("&[{}]", from), to)
                })
                .split(" ")
                .filter(|x| *x != "")
                .collect::<Vec<_>>(),
        )
        .kill_on_drop(true)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
}
