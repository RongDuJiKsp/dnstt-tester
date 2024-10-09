use crate::common::child::load_env_and_run;
use anyhow::anyhow;
use clap::Parser;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::process::Child;

#[derive(Parser)]
struct ServerArgs {
    //side 是需要运行的端点，可以是client or server
    side: String,
    //dnstt 将要执行的端口号
    #[arg(short, long)]
    port: u16,
    //dnstt 可执行文件名称，接受一个参数，为端口号
    #[arg(short, long)]
    exe: String,
    //dnstt 可执行文件参数，端口号可用 &[port] 代替
    #[arg(short, long, allow_hyphen_values = true)]
    args: String,
}
async fn new_server(args: &ServerArgs) -> anyhow::Result<Child> {
    load_env_and_run(&args.exe, &args.args, args.port)
        .map_err(|e| anyhow!("Failed To Run Server Because {}", e))
}
async fn loop_read(mut stream: TcpStream) {
    let mut buf = [0u8; 1024];
    loop {
        match stream.read(&mut buf).await {
            Ok(size) => {
                if size == 0 {
                    println!("Client Read Close");
                    return;
                }
            }
            Err(e) => {
                println!("Client Read Err:{}", e);
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
        println!("dnstt Server Created");
        loop {
            let w = server.wait().await;
            println!("dnstt exited because {:#?}", w);
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
        println!("New Client Conn :{}", addr);
        tokio::spawn(async move {
            loop_read(stream).await;
            println!("Connection closed: {:?}", addr);
        });
    }
}
