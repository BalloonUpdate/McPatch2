use std::collections::LinkedList;
use std::time::SystemTime;

use serde::ser::SerializeMap;
use serde::Serialize;

pub const MAX_LOGS: usize = 5000;

pub struct LogEntry {
    pub time: SystemTime,
    pub content: String,
    pub level: LogLevel,
}

impl Serialize for LogEntry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        let unix_ts = self.time.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();

        let mut map = serializer.serialize_map(Some(3))?;
        map.serialize_entry("time", &unix_ts)?;
        map.serialize_entry("content", &self.content)?;
        map.serialize_entry("level", &self.level)?;
        map.end()
    }
}

#[derive(Clone, Copy)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

impl Serialize for LogLevel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        let text = match self {
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warning => "warning",
            LogLevel::Error => "error",
        };

        serializer.collect_str(text)
    }
}


#[derive(Clone)]
pub struct Line {
    pub read: bool,
    pub time: SystemTime,
    pub content: String,
    pub level: LogLevel,
}

impl Line {
    pub fn new(content: String, level: LogLevel) -> Self {
        Self {
            read: false,
            time: SystemTime::now(),
            content,
            level,
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

    pub fn get_logs<'a>(&'a mut self, full: bool) -> Vec<LogEntry> {
        let mut entries = Vec::<LogEntry>::new();

        if full {
            for log in &mut self.buf {
                log.read = true;
            }

            for line in &self.buf {
                entries.push(LogEntry { time: line.time, content: line.content.to_owned(), level: line.level.clone() });
            }
        } else {
            for line in &self.buf {
                if !line.read {
                    entries.push(LogEntry { time: line.time, content: line.content.to_owned(), level: line.level.clone() });
                }
            }

            for log in &mut self.buf {
                log.read = true;
            }
        }

        entries
    }

    pub fn log_debug(&mut self, content: impl AsRef<str>) {
        self.log(content, LogLevel::Debug);
    }

    pub fn log_info(&mut self, content: impl AsRef<str>) {
        self.log(content, LogLevel::Info);
    }

    pub fn log_warning(&mut self, content: impl AsRef<str>) {
        self.log(content, LogLevel::Warning);
    }

    pub fn log_error(&mut self, content: impl AsRef<str>) {
        self.log(content, LogLevel::Error);
    }

    fn log(&mut self, content: impl AsRef<str>, level: LogLevel) {
        for line in content.as_ref().split("\n") {
            println!("{}", line);

            self.buf.push_back(Line::new(line.to_owned(), level));

            while self.buf.len() > MAX_LOGS {
                self.buf.pop_front();
            }
        }
    }
}