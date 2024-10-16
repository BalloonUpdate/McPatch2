//! 运行内置服务端，使用私有协议
use std::io::ErrorKind;
use std::net::TcpListener;
use std::ops::Range;
use std::time::SystemTime;

use chrono::Local;
use shared::utility::partial_read::PartialAsyncRead;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncSeekExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use crate::utility::traffic_control::AsyncTrafficControl;
use crate::AppContext;

pub fn do_serve(port: u16, ctx: &AppContext) -> i32 {
    println!("==============改动说明==============");
    println!("自管理端0.0.14版本开始");
    println!("serve命令的端口设置，已经从命令行参数，移动到配置文件config.toml中");
    println!("请备份现有的config.toml后，删除此文件重新生成");
    println!("============================");

    let capacity = ctx.config.serve_tbf_burst;
    let regain = ctx.config.serve_tbf_rate;

    if capacity > 0 && regain > 0 {
        println!("启动内置服务端，限速参数：突发容量：{}, 限速速率：{}", capacity, regain);
    } else {
        println!("启动内置服务端");
    }

    let host = &ctx.config.serve_listen_addr;
    let port = format!("{}", if port != 0 { port } else { ctx.config.serve_listen_port });

    let listen_ip_port = format!("{}:{}", host, port);

    let listener = TcpListener::bind(listen_ip_port).unwrap();
    println!("listening on {}:{}", host, port);

    let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        stream.set_read_timeout(Some(std::time::Duration::from_secs(30))).expect("设置连接 Read Timeout 参数失败");
        stream.set_write_timeout(Some(std::time::Duration::from_secs(30))).expect("设置连接 Write Timeout 参数失败");
        let ctx = ctx.clone();
        runtime.spawn(async move { serve_loop(stream, ctx).await });
    }

    0
}

async fn serve_loop(stream: std::net::TcpStream, ctx: AppContext) {
    let mut stream = TcpStream::from_std(stream).unwrap();

    async fn inner(mut stream: &mut TcpStream, ctx: &AppContext, info: &mut Option<(String, Range<u64>)>) -> std::io::Result<()> {
        // 接收文件路径
        let mut path = String::with_capacity(1024);
        receive_data(&mut stream).await?.read_to_string(&mut path).await?;

        let start = stream.read_u64_le().await?;
        let mut end = stream.read_u64_le().await?;

        *info = Some((path.to_owned(), start..end));

        let path = ctx.public_dir.join(path);

        assert!(start <= end, "the end is {} and the start is {}", end, start);

        // 检查文件大小
        let len = match tokio::fs::metadata(&path).await {
            Ok(meta) => {
                // 请求的范围不对，返回-2
                if end > meta.len() {
                    stream.write_all(&(-2i64).to_le_bytes()).await?;
                    return Ok(());
                }
                meta.len()
            },
            Err(_) => {
                // 文件没有找到，返回-1
                stream.write_all(&(-1i64).to_le_bytes()).await?;
                return Ok(());
            },
        };

        // 如果不指定范围就发送整个文件
        if start == 0 && end == 0 {
            end = len as u64;
        }

        let mut remains = end - start;

        // 文件已经找到，发送文件大小
        stream.write_all(&(remains as i64).to_le_bytes()).await?;

        // 传输文件内容
        let mut file = tokio::fs::File::open(path).await?;
        file.seek(std::io::SeekFrom::Start(start)).await?;

        // 增加限速效果
        let mut file = AsyncTrafficControl::new(&mut file, ctx.config.serve_tbf_burst as u64, ctx.config.serve_tbf_rate as u64);

        while remains > 0 {
            let mut buf = [0u8; 32 * 1024];
            let limit = buf.len().min(remains as usize);
            let buf = &mut buf[0..limit];
            
            let read = file.read(buf).await?;
            
            stream.write_all(&buf[0..read]).await?;

            remains -= read as u64;

            // tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        Ok(())
    }

    loop {
        let mut info = Option::<(String, Range<u64>)>::None;

        let start = SystemTime::now();
        let result = inner(&mut stream, &ctx, &mut info).await;
        let time = SystemTime::now().duration_since(start).unwrap();

        match result {
            Ok(_) => {
                // 既然result是ok，那么info一定不是none
                let info = info.unwrap();
                let ts = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                println!("[{}] {} - {} {}+{} ({}ms)", ts, stream.peer_addr().unwrap(), info.0, info.1.start, info.1.end - info.1.start, time.as_millis());
            },
            Err(e) => {
                match e.kind() {
                    ErrorKind::UnexpectedEof => {},
                    ErrorKind::ConnectionAborted => {},
                    ErrorKind::ConnectionReset => {},
                    _ => println!("{} - {:?}", stream.peer_addr().unwrap(), e.kind()),
                }
    
                break;
            },
        }
    }
}

async fn _send_data(stream: &mut TcpStream, data: &[u8]) -> std::io::Result<()> {
    stream.write_u64_le(data.len() as u64).await?;
    stream.write_all(data).await?;

    Ok(())
}

async fn receive_data<'a>(stream: &'a mut TcpStream) -> std::io::Result<PartialAsyncRead<'a, TcpStream>> {
    let len = stream.read_u64_le().await?;

    Ok(PartialAsyncRead::new(stream, len))
}