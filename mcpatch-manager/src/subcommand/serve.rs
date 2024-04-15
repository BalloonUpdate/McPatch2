//! 运行内置服务端，使用私有协议

use std::net::TcpListener;

use mcpatch_shared::utility::limited_read_async::LimitedReadAsync;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncSeekExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use crate::AppContext;

pub fn do_serve(_ctx: &AppContext) -> i32 {
    println!("启动内置服务端");

    let host = "0.0.0.0";
    let port = "660000";

    let listener = TcpListener::bind(format!("{}:{}", host, port)).unwrap();
    println!("listening on {}:{}", host, port);

    let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let stream = TcpStream::from_std(stream).unwrap();

        let ctx = _ctx.clone();
        runtime.spawn(async move { serve_loop(stream, ctx).await });
    }

    0
}

async fn serve_loop(mut stream: TcpStream, ctx: AppContext) {
    let mut buf = Vec::<u8>::with_capacity(1024 * 1024);

    loop {
        // 接收文件路径
        let path = std::str::from_utf8(receive_data(&mut stream, &mut buf).await.unwrap()).unwrap();
        let path = ctx.public_dir.join(path);

        let start = receive_data(&mut stream, &mut buf).await.unwrap().read_u64_le().await.unwrap();
        let end = receive_data(&mut stream, &mut buf).await.unwrap().read_u64_le().await.unwrap();

        assert!(end > start);

        // 检查文件大小
        match tokio::fs::metadata(&path).await {
            Ok(meta) => {
                // 请求的范围不对劲
                if end > meta.len() {
                    send_data(&mut stream, &[2]).await.unwrap();
                    continue;
                }
            },
            Err(_) => {
                // 文件没有找到
                send_data(&mut stream, &[1]).await.unwrap();
                continue;
            },
        };

        // 文件已经找到
        send_data(&mut stream, &[1]).await.unwrap();

        let mut file = tokio::fs::File::open(path).await.unwrap();
        file.seek(std::io::SeekFrom::Start(start)).await.unwrap();

        let mut remains = (end - start) as usize;

        while remains > 0 {
            buf.clear();

            let read = file.read(&mut buf).await.unwrap();
            
            stream.write_all(&buf[0..read]).await.unwrap();

            remains -= read.min(remains);
        }
        
    }
}

async fn send_data(stream: &mut TcpStream, data: &[u8]) -> std::io::Result<()> {
    stream.write_u64_le(data.len() as u64).await?;
    stream.write_all(data).await?;

    Ok(())
}

async fn receive_data<'a>(stream: &mut TcpStream, buf: &'a mut Vec<u8>) -> std::io::Result<&'a [u8]> {
    buf.clear();
    
    let len = stream.read_u64_le().await?;
    let recv = LimitedReadAsync::new(stream, len).read_to_end(buf).await.unwrap();

    Ok(&buf[0..recv])
}