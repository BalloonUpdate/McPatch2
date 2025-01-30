use std::io::Read;

use tokio::io::AsyncRead;
use tokio::pin;

/// 代表一个限制读取数量的Read
pub struct PartialRead<R: Read>(R, u64);

impl<R: Read> PartialRead<R> {
    pub fn new(read: R, count: u64) -> Self {
        Self(read, count)
    }
}

impl<R: Read> Read for PartialRead<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.1 == 0 {
            return Ok(0);
        }
        let consume = self.1.min(buf.len() as u64);
        self.1 -= consume;
        self.0.read(&mut buf[0..consume as usize])
    }
}

/// 代表一个限制读取数量的Read
pub struct PartialAsyncRead<R: AsyncRead>(R, u64);

impl<R: AsyncRead> PartialAsyncRead<R> {
    pub fn new(read: R, count: u64) -> Self {
        Self(read, count)
    }

    pub fn count(&self) -> u64 {
        self.1
    }
}

impl<R: AsyncRead + Unpin> AsyncRead for PartialAsyncRead<R> {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        if self.1 == 0 {
            return std::task::Poll::Ready(Ok(()));
        }

        let limit = buf.remaining().min(self.1 as usize);
        let partial = &mut buf.take(limit);

        let read = &mut self.0;
        pin!(read);

        match read.poll_read(cx, partial) {
            std::task::Poll::Ready(ready) => {
                let adv = partial.filled().len();

                unsafe { buf.assume_init(adv); }
                buf.advance(adv);

                self.1 -= adv as u64;

                std::task::Poll::Ready(ready)
            },
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio::io::AsyncReadExt;

    use crate::utility::partial_read::PartialAsyncRead;

    #[test]
    fn partial_async_read_test() {
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    
        runtime.block_on(async {
            const TOTAL: usize = 884;
            const BUF: usize = 1000;
            
            let mut data = Vec::<u8>::new();

            for i in 0..TOTAL {
                data.push((i % u8::MAX as usize) as u8);
            }

            let mut expectation = data.clone();
            expectation.reverse();

            let mut data = &data[..];
    
            let count = data.len();
            let mut partial = PartialAsyncRead::new(&mut data, count as u64);

            let mut buf = [0u8; BUF];

            loop {
                let read = partial.read(&mut buf).await.unwrap();

                if read == 0 {
                    break;
                }

                for b in &buf[0..read] {
                    assert_eq!(b, &expectation.pop().unwrap());
                }
            }

            assert!(expectation.is_empty());
        });
    }
}