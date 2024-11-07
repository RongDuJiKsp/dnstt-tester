use crate::common::child::{bind_client_to_files, bind_half_to_files, run_exe_with_env};
use crate::common::log::Log;
use crate::common::random::RandomPacker;
use crate::common::sync::PtrFac;
use crate::common::timer::Timer;
use anyhow::anyhow;
use clap::Parser;
use std::collections::HashMap;
use std::io::SeekFrom;
use std::str::FromStr;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::process::Child;
use tokio::select;
use tokio::time::sleep;

#[derive(Debug, Parser)]
struct ClientArgs {
    //side 是需要运行的端点，可以是client or server
    side: String,
    //客户端程序 将要绑定的端口号
    #[arg(short, long)]
    port: u16,
    // dnstt 客户端程序将要绑定的ip
    #[arg(long, default_value = "127.0.0.1")]
    bind: String,
    //dnstt 可执行文件名称，接受一个参数，为端口号
    #[arg(short, long)]
    exe: String,
    //dnstt 可执行文件参数，端口号可用 &[port] 代替
    #[arg(short, long, allow_hyphen_values = true)]
    args: String,
    //定时切断连接的时间
    #[arg(short, long)]
    reconnect_time_second: u64,
    //定时重启连接的时间
    #[arg(short, long)]
    conn_time_second: u64,
    //定时发送随机文件的时间间隔
    #[arg(short, long)]
    make_file_second: u64,
    //发送文件的大小范围，格式为xx~xx 单位为字节
    #[arg(short, long)]
    file_size_range: String,
    //需要写入执行文件 stdin的文件
    #[arg(long = "in")]
    stdin_file: String,
    //转储stdout的文件
    #[arg(long = "out")]
    stdout_file: String,
    //转储stderr的文件
    #[arg(long = "err")]
    stderr_file: String,
    #[arg(long = "no_stdin")]
    no_stdin: bool,
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
type ProcessCtx = (Child, TcpStream);
//client端 启动dnstt client并且主动发起连接
async fn create_dnstt_client_and_tcp_conn(args: &ClientArgs) -> anyhow::Result<ProcessCtx> {
    let child = run_exe_with_env(
        &args.exe,
        &args.args,
        &HashMap::from([(format!("{}", "port"), format!("{}", args.port))]),
    )
        .map_err(|e| anyhow!("Failed to create dnstt client :{}", e))?;
    sleep(Duration::from_secs(2)).await;
    let tcp = TcpStream::connect(format!("{}:{}", args.bind, args.port))
        .await
        .map_err(|e| anyhow!("Failed to create tcp conn :{}", e))?;
    println!("start client and conn successfully");
    Ok((child, tcp))
}

async fn reconnect(ctx: &mut ProcessCtx, arg: &ClientArgs) -> anyhow::Result<()> {
    let (client, stream) = ctx;
    println!("Shutdown Client ing...");
    stream.shutdown().await?;
    let _ = client.kill().await;
    let _ = client.wait().await;
    println!("Waiting To Restart Client");
    sleep(Duration::from_secs(arg.conn_time_second)).await;
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
    let mut pcx = create_dnstt_client_and_tcp_conn(&arg).await.unwrap();
    if arg.no_stdin {
        bind_half_to_files(&mut pcx.0, file_stdout.clone(), file_stderr.clone());
    } else {
        bind_client_to_files(
            &mut pcx.0,
            file_stdin.clone(),
            file_stdout.clone(),
            file_stderr.clone(),
        );
    }
    let mut make_file_timer = Timer::timer(Duration::from_secs(arg.make_file_second));
    let mut reconnect_timer = Timer::timer(Duration::from_secs(arg.reconnect_time_second));
    loop {
        select! {
            _=make_file_timer.tick()=>{
                let r= send_file(&mut pcx.1,&mut rand).await;
                println!("tick to make file");
                Log::error_if_err(r);
            }
            _=reconnect_timer.tick()=>{
                let r= reconnect(&mut pcx,& arg).await;
                let t= file_stdin.lock().await.seek(SeekFrom::Start(0)).await;
                Log::error_if_err(r);
                Log::error_if_err(t);
                if arg.no_stdin {
                     bind_half_to_files(&mut pcx.0, file_stdout.clone(), file_stderr.clone());
                } else {
                     bind_client_to_files(
                     &mut pcx.0,
                     file_stdin.clone(),
                     file_stdout.clone(),
                     file_stderr.clone(),
                );
                }
                println!("tick to restart");
            }
        }
    }
}
