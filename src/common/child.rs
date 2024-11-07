use crate::common::stdio::TransferStdio;
use crate::common::sync::{PtrFac, Shared};
use std::collections::HashMap;
use std::process::Stdio;
use tokio::io;
use tokio::io::{AsyncRead, AsyncWrite};
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
pub fn bind_client_to_files<
    R: AsyncRead + Unpin + Send + 'static,
    W: AsyncWrite + Unpin + Send + 'static,
>(
    client: &mut Child,
    stdin: Shared<R>,
    stdout: Shared<W>,
    stderr: Shared<W>,
) {
    let (i, o, e) = (
        client.stdin.take().expect("Stdin Is Err Can't Logger"),
        client.stdout.take().expect("Stdout Is Err Can't Logger"),
        client.stderr.take().expect("Stderr Is Err Can't Logger"),
    );
    TransferStdio::spawn_copy(PtrFac::share(i), stdin);
    TransferStdio::spawn_copy(stdout, PtrFac::share(o));
    TransferStdio::spawn_copy(stderr, PtrFac::share(e));
}
pub fn bind_half_to_files<W: AsyncWrite + Unpin + Send + 'static>(
    client: &mut Child,
    stdout: Shared<W>,
    stderr: Shared<W>,
) {
    let (o, e) = (
        client.stdout.take().expect("Stdout Is Err Can't Logger"),
        client.stderr.take().expect("Stderr Is Err Can't Logger"),
    );
    TransferStdio::spawn_copy(stdout, PtrFac::share(o));
    TransferStdio::spawn_copy(stderr, PtrFac::share(e));
}
