use std::collections::LinkedList;
use std::time::SystemTime;

pub const MAX_LOGS: usize = 5000;

#[derive(Clone)]
pub struct Line {
    pub read: bool,
    pub time: SystemTime,
    pub content: String,
}

impl Line {
    pub fn new(content: String) -> Self {
        Self {
            read: false,
            time: SystemTime::now(),
            content,
        }
    }
}

pub struct ConsoleBuffer {
    buf: LinkedList<Line>,
}

impl ConsoleBuffer {
    pub fn new() -> Self {
        ConsoleBuffer {
            buf: LinkedList::new(),
        }
    }

    pub fn get_logs<'a>(&'a mut self, full: bool) -> Vec<Line> {
        if full {
            for log in &mut self.buf {
                log.read = true;
            }
    
            self.buf.iter().map(|e| e.to_owned()).collect()
        } else {
            let logs = self.buf.iter().filter(|e| !e.read).map(|e| e.to_owned()).collect();

            for log in &mut self.buf {
                log.read = true;
            }

            logs
        }
    }

    pub fn log(&mut self, content: impl AsRef<str>) {
        let content = content.as_ref().to_string();

        println!("{}", content);

        self.buf.push_back(Line::new(content));

        while self.buf.len() > MAX_LOGS {
            self.buf.pop_front();
        }
    }
}