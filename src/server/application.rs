use crate::common::child::run_exe_with_env;
use anyhow::anyhow;
use clap::Parser;
use std::collections::HashMap;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::process::Child;
use tokio::time::sleep;

#[derive(Parser, Debug)]
struct ServerArgs {
    //side 是需要运行的端点，可以是client or server
    side: String,
    //隧道测试工具 将要监听的端口号
    #[arg(short, long)]
    port: u16,
    //隧道工具可执行文件名称
    #[arg(short, long)]
    exe: String,
    //隧道工具可执行文件参数，端口号可用 &[port] 代替
    #[arg(short, long, allow_hyphen_values = true)]
    args: String,
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
pub async fn run_application() {
    let arg = ServerArgs::parse();
    let tcp = TcpListener::bind(&format!("127.0.0.1:{}", arg.port))
        .await
        .unwrap();
    tokio::spawn(async move {
        let mut server = new_server(&arg).await.unwrap();
        println!("Tunnel Server Created");
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
            }
        }
    });
    loop {
        let (stream, addr) = match tcp.accept().await {
            Ok(e) => e,
            Err(_) => continue,
        };
        println!("New Client Conn Addr :{}", addr);
        tokio::spawn(async move {
            loop_read(stream).await;
            println!("Connection Addr {} closed", addr);
        });
    }
}
