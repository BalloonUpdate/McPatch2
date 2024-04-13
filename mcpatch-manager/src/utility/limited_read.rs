use std::io::Read;

/// 代表一个限制读取数量的Read
pub struct LimitedRead<'a, R: Read>(&'a mut R, u64);

impl<'a, R: Read> LimitedRead<'a, R> {
    pub fn new(raw_read: &'a mut R, limit: u64) -> Self {
        Self(raw_read, limit)
    }
}

impl<R: Read> Read for LimitedRead<'_, R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.1 == 0 {
            return Ok(0);
        }
        let consume = self.1.min(buf.len() as u64);
        self.1 -= consume;
        self.0.read(&mut buf[0..consume as usize])
    }
}