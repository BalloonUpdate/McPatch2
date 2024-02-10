use std::io::Write;
use std::ops::Deref;
use std::ops::DerefMut;

pub struct CountedWrite<W: Write>(pub W, u64);

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

impl<W: Write> Deref for CountedWrite<W> {
    type Target = W;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<W: Write> DerefMut for CountedWrite<W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}