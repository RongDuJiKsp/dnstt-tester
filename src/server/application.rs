use crate::common::child::{bind_client_to_files, run_exe_with_env};
use crate::common::sync::PtrFac;
use anyhow::anyhow;
use clap::Parser;
use std::collections::HashMap;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::process::Child;
use tokio::task::JoinHandle;
use tokio::time::sleep;

#[derive(Parser, Debug)]
struct ServerArgs {
    //side 是需要运行的端点，可以是client or server
    side: String,
    //隧道测试工具 将要为可执行文件参数的端口号
    #[arg(short, long)]
    port: u16,
    //隧道测试工具将要监听的端口号组，用 "," 分割
    #[arg(long, default_value = "")]
    ports: String,
    //隧道工具可执行文件名称
    #[arg(short, long)]
    exe: String,
    // dnstt 服务程序将要绑定的ip
    #[arg(long, default_value = "127.0.0.1")]
    bind: String,
    //隧道工具可执行文件参数，端口号可用 &[port] 代替
    #[arg(short, long, allow_hyphen_values = true)]
    args: String,
    //需要写入执行文件 stdin的文件
    #[arg(long = "in")]
    stdin_file: String,
    //转储stdout的文件
    #[arg(long = "out")]
    stdout_file: String,
    //转储stderr的文件
    #[arg(long = "err")]
    stderr_file: String,
}
async fn new_server(args: &ServerArgs) -> anyhow::Result<Child> {
    run_exe_with_env(
        &args.exe,
        &args.args,
        &HashMap::from([(format!("{}", "port"), format!("{}", args.port))]),
    )
    .map_err(|e| anyhow!("Failed To Run Server Because {}", e))
}
async fn loop_read(mut stream: TcpStream) {
    let mut buf = [0u8; 1024];
    loop {
        match stream.read(&mut buf).await {
            Ok(size) => {
                if size == 0 {
                    println!("Client Pipe Close");
                    return;
                }
            }
            Err(e) => {
                println!("Client Read With Err:{}", e);
                return;
            }
        }
    }
}
pub async fn listen_ports(i: &str, p: &str) -> Vec<TcpListener> {
    let mut conns = Vec::new();
    for port in p.split(",") {
        conns.push(TcpListener::bind(&format!("{}:{}", i, port)).await.unwrap())
    }
    conns
}
pub async fn run_application() {
    let arg = ServerArgs::parse();
    let conn = listen_ports(
        &arg.bind,
        &if arg.ports.is_empty() {
            format!("{}", arg.port)
        } else {
            arg.ports.clone()
        },
    )
    .await;
    let file_stdin = PtrFac::share(
        File::options()
            .read(true)
            .open(&arg.stdin_file)
            .await
            .unwrap(),
    );
    let file_stdout = PtrFac::share(
        File::options()
            .write(true)
            .open(&arg.stdout_file)
            .await
            .unwrap(),
    );
    let file_stderr = PtrFac::share(
        File::options()
            .write(true)
            .open(&arg.stderr_file)
            .await
            .unwrap(),
    );
    tokio::spawn(async move {
        let mut server = new_server(&arg).await.unwrap();
        println!("Tunnel Server Created");
        bind_client_to_files(
            &mut server,
            file_stdin.clone(),
            file_stdout.clone(),
            file_stderr.clone(),
        );
        loop {
            match server.wait().await {
                Ok(e) => println!("Server Exited with code {}", e.code().unwrap_or_default()),
                Err(e) => println!("Server Exited with err :{}", e),
            }
            sleep(Duration::from_secs(4)).await;
            println!("Server restarting");
            server = match new_server(&arg).await {
                Ok(e) => e,
                Err(e) => {
                    println!("Start DNSTT Server fail :{},Retrying", e);
                    continue;
                }
            };
            bind_client_to_files(
                &mut server,
                file_stdin.clone(),
                file_stdout.clone(),
                file_stderr.clone(),
            );
        }
    });
    let mut hds: Vec<JoinHandle<()>> = Vec::new();
    for con in conn {
        let hd = tokio::spawn(async move {
            loop {
                let (stream, addr) = match con.accept().await {
                    Ok(e) => e,
                    Err(_) => break,
                };
                println!("New Client Conn Addr :{}", addr);
                tokio::spawn(async move {
                    loop_read(stream).await;
                    println!("Connection Addr {} closed", addr);
                });
            }
        });
        hds.push(hd);
    }
    for hd in hds {
        let _ = hd.await;
    }
}
