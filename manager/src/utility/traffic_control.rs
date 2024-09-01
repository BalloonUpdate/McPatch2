use std::task::Poll;
use std::time::Duration;
use std::time::SystemTime;

use once_cell::sync::Lazy;
use tokio::io::AsyncRead;

/// 专门给`AsyncTrafficControl`用的runtime，用来再令牌不够的情况下，延时唤醒waker
static TASK_WAKER_RUNTIME: Lazy<tokio::runtime::Runtime> = Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .thread_name("async tc waker")
        .enable_all()
        .worker_threads(1)
        .build()
        .unwrap()
});

/// 基于令牌桶（Token Bucket）的简单流量控制（Token Bucket Filter）
pub struct AsyncTrafficControl<'a, R: AsyncRead + Unpin> {
    /// 源数据
    read: &'a mut R,

    /// 桶中令牌的数量
    bucket: u64,

    /// 桶的容量
    capacity: u64,

    /// 每毫秒往桶中增加多少令牌
    rate_per_ms: u64,

    /// 上传调用的时间
    last_time: SystemTime,
}

impl<'a, R: AsyncRead + Unpin> AsyncTrafficControl<'a, R> {
    pub fn new(read: &'a mut R, capacity: u64, rate_per_second: u64) -> Self {
        Self { 
            read, 
            bucket: 0, 
            capacity, 
            rate_per_ms: rate_per_second / 1000, 
            last_time: SystemTime::now(),
        }
    }
}

impl<'a, R: AsyncRead + Unpin> AsyncRead for AsyncTrafficControl<'a, R> {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        // 如果参数设置为0，则不经过限速直接读取
        if self.rate_per_ms == 0 || self.capacity == 0 {
            let read = &mut self.read;
            tokio::pin!(read);
            return read.poll_read(cx, buf);
        }

        // 循环等待，直到有足够的令牌
        let consumption = {
            let now = SystemTime::now();
            let dt = now.duration_since(self.last_time).unwrap();

            // 将检测粒度增加到1ms以上。不然因为整数的原因，当dt.as_millis()小于0时，new_tokens永远是0
            if dt.as_millis() > 0 {
                self.last_time = now;

                let new_tokens = dt.as_millis() as u64 * self.rate_per_ms;
    
                self.bucket += new_tokens;
                self.bucket = self.bucket.min(self.capacity);
            }
            
            // 计算本次能消耗掉的令牌数
            let consumption = self.bucket.min(buf.remaining() as u64);
            
            // println!("bucket: {} | com: {}", self.bucket, consumption);

            match consumption > 0 {
                true => consumption,
                false => {
                    // println!("set");

                    // 如果令牌用完了，就需要等100毫秒再唤醒，好等待新的令牌过来
                    let waker = cx.waker().clone();
                    TASK_WAKER_RUNTIME.spawn(async move {
                        // println!("a");
                        tokio::time::sleep(Duration::from_millis(100)).await;
                        // println!("b");
                        waker.wake();
                    });

                    return Poll::Pending;
                },
            }
        };

        // println!("+ bucket: {}, consumption: {}", self.bucket, consumption);

        // 按consumption取出一小部分缓冲区
        let mut new_buf = buf.take(consumption as usize);

        // pin住read对象
        let read = &mut self.read;
        tokio::pin!(read);

        // 进行读取
        match read.poll_read(cx, &mut new_buf) {
            Poll::Ready(_) => (),
            Poll::Pending => return Poll::Pending,
        }

        // 读取成功后，消耗掉这些令牌
        let consumption = new_buf.filled().len() as u64;

        // 上面的buf.take()返回的是一个新的buf，往这个新的buf里写东西，buf本身是感知不到的
        // 所以这里需要手动推进一下缓冲区指针
        buf.advance(consumption as usize);

        self.bucket -= consumption;
        
        Poll::Ready(Ok(()))
    }
}