use tokio::io::AsyncRead;
use tokio::pin;

/// 代表一个限制读取数量的Read
pub struct LimitedReadAsync<'a, R: AsyncRead>(&'a mut R, u64);

impl<'a, R: AsyncRead> LimitedReadAsync<'a, R> {
    pub fn new(raw_read: &'a mut R, limit: u64) -> Self {
        Self(raw_read, limit)
    }

    pub fn len(&self) -> u64 {
        self.1
    }
}

impl<R: AsyncRead + Unpin> AsyncRead for LimitedReadAsync<'_, R> {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        if self.1 == 0 {
            return std::task::Poll::Ready(Ok(()));
        }

        let consume = self.1.min(buf.capacity() as u64);
        self.1 -= consume;

        let read = &mut self.0;
        pin!(read);
        read.poll_read(cx, &mut buf.take(consume as usize))
    }
}