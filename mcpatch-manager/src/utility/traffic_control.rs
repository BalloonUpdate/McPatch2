use std::io::Read;
use std::time::Duration;
use std::time::SystemTime;

/// 基于令牌桶（Token Bucket）的简单流量控制（Token Bucket Filter）
pub struct TrafficControl<'a, R: Read> {
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

impl<'a, R: Read> TrafficControl<'a, R> {
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

impl<'a, R: Read> Read for TrafficControl<'a, R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.rate_per_ms == 0 || self.capacity == 0 {
            return self.read.read(buf);
        }

        let consumption = loop {
            let now = SystemTime::now();
            let dt = now.duration_since(self.last_time).unwrap();
            self.last_time = now;

            let new_tokens = dt.as_millis() as u64 * self.rate_per_ms;

            self.bucket += new_tokens;
            self.bucket = self.bucket.min(self.capacity);

            let consumption = self.bucket.min(buf.len() as u64);

            if consumption > 0 {
                break consumption;
            }

            std::thread::sleep(Duration::from_millis(50));
        };

        let result = self.read.read(&mut buf[0..consumption as usize]);

        let consumption = match &result {
            Ok(read) => *read as u64,
            Err(_) => 0,
        };

        self.bucket -= consumption;
        
        result
    }
}