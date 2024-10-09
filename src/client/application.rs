use crate::common::log::Log;
use crate::common::random::RandomPacker;
use crate::common::timer::Timer;
use anyhow::anyhow;
use clap::Parser;
use std::process::Stdio;
use std::str::FromStr;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::process::{Child, Command};
use tokio::select;
use tokio::time::sleep;

#[derive(Debug, Parser)]
struct ClientArgs {
    //side 是需要运行的端点，可以是client or server
    side: String,
    //dnstt 将要执行的端口号
    #[arg(short, long)]
    port: u16,
    //dnstt 运行脚本，接受一个参数，为端口号
    #[arg(short, long)]
    shell: String,
    //定时切断连接的时间
    #[arg(short, long)]
    reconnect_time_second: u64,
    //定时发送随机文件的时间间隔
    #[arg(short, long)]
    make_file_second: u64,
    //发送文件的大小范围，格式为xx~xx 单位为字节
    #[arg(short, long)]
    file_size_range: String,
}
impl ClientArgs {
    fn file_size(&self) -> (u64, u64) {
        let mut i = self.file_size_range.split("~");
        let left_size: u64 = u64::from_str(i.next().expect("文件大小格式不对！")).unwrap();
        let right_size: u64 = u64::from_str(i.next().expect("文件大小格式不对！")).unwrap();
        if left_size < right_size {
            (left_size, right_size)
        } else {
            (right_size, left_size)
        }
    }
}
//client端 启动dnstt client并且主动发起连接
async fn create_dnstt_client_and_tcp_conn(arg: &ClientArgs) -> anyhow::Result<(Child, TcpStream)> {
    let child = Command::new("sh")
        .arg(arg.shell.clone())
        .arg(arg.port.to_string())
        .kill_on_drop(true)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| anyhow!("Failed to create dnstt client :{}", e))?;
    sleep(Duration::from_secs(2)).await;
    let tcp = TcpStream::connect(format!("127.0.0.1:{}", arg.port))
        .await
        .map_err(|e| anyhow!("Failed to create tcp conn :{}", e))?;
    println!("start client and conn successfully");
    Ok((child, tcp))
}
async fn reconnect(
    client: &mut Child,
    stream: &mut TcpStream,
    arg: &ClientArgs,
) -> anyhow::Result<()> {
    let (c, t) = create_dnstt_client_and_tcp_conn(arg).await?;
    *client = c;
    *stream = t;
    Ok(())
}
async fn send_file(stream: &mut TcpStream, rand: &mut RandomPacker) -> anyhow::Result<()> {
    stream.write_all(&rand.random_bytes()).await?;
    Ok(())
}
pub async fn run_application() {
    let arg = ClientArgs::parse();
    let (m_in, m_ax) = arg.file_size();
    let mut rand = RandomPacker::new(m_in, m_ax);
    let (mut client, mut stream) = create_dnstt_client_and_tcp_conn(&arg).await.unwrap();
    let mut make_file_timer = Timer::timer(Duration::from_secs(arg.make_file_second));
    let mut reconnect_timer = Timer::timer(Duration::from_secs(arg.reconnect_time_second));
    loop {
        select! {
            _=make_file_timer.tick()=>{
                let r= send_file(&mut stream,&mut rand).await;
                println!("tick to make file");
                Log::error_if_err(r);
            }
            _=reconnect_timer.tick()=>{
                let r= reconnect(&mut client,&mut stream,& arg).await;
                println!("tick to restart");
                Log::error_if_err(r);
            }
        }
    }
}
