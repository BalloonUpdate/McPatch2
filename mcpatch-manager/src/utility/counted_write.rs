use std::io::Write;

/// 代表一个计数的Write对象
pub struct CountedWrite<W: Write>(W, u64);

impl<W: Write> CountedWrite<W> {
    pub fn new(write: W) -> Self {
        Self(write, 0)
    }

    pub fn count(&self) -> u64 {
        self.1
    }
}

impl<W: Write> Write for CountedWrite<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self.0.write(buf) {
            Ok(count) => {
                self.1 += count as u64;
                Ok(count)
            },
            Err(e) => Err(e),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
}

// impl<W: Write> Deref for CountedWrite<W> {
//     type Target = W;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl<W: Write> DerefMut for CountedWrite<W> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.0
//     }
// }

#[cfg(test)]
mod tests {
    use std::io::Read;

    use crate::utility::counted_write::CountedWrite;

    #[test]
    fn test_counted_writer() {
        let count = 1024 * 1024 * 64;

        let mut src = std::io::repeat(0).take(count);
        let mut dst = CountedWrite::new(std::io::sink());

        let copied = std::io::copy(&mut src, &mut dst).unwrap();

        assert_eq!(copied, count);
    }
}