use std::collections::LinkedList;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::SystemTime;

use serde::ser::SerializeMap;
use serde::Serialize;

pub const MAX_LOGS: usize = 5000;

/// 代表一个日志缓冲区。负责收集各种任务运行中的输出
#[derive(Clone)]
pub struct Console {
    inner: Arc<Mutex<Inner>>,
}

impl Console {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner { buf: LinkedList::new() }))
        }
    }

    /// 获取目前的日志。
    /// 
    /// + 若`full`为true，则获取所有的日志
    /// + 若`full`为false，则获取从上次调用此方法以来的新产生的日志
    pub fn get_logs<'a>(&'a self, full: bool) -> Vec<LogOutputed> {
        let mut lock = self.inner.lock().unwrap();

        let mut entries = Vec::<LogOutputed>::new();

        if full {
            for log in &mut lock.buf {
                log.read = true;
            }

            for line in &lock.buf {
                entries.push(LogOutputed { time: line.time, content: line.content.to_owned(), level: line.level.clone() });
            }
        } else {
            for line in &lock.buf {
                if !line.read {
                    entries.push(LogOutputed { time: line.time, content: line.content.to_owned(), level: line.level.clone() });
                }
            }

            for log in &mut lock.buf {
                log.read = true;
            }
        }

        entries
    }

    /// 记录一条“调试”日志
    pub fn log_debug(&self, content: impl AsRef<str>) {
        self.log(content, LogLevel::Debug);
    }

    /// 记录一条“普通”日志
    pub fn log_info(&self, content: impl AsRef<str>) {
        self.log(content, LogLevel::Info);
    }

    /// 记录一条“警告”日志
    pub fn log_warning(&self, content: impl AsRef<str>) {
        self.log(content, LogLevel::Warning);
    }

    /// 记录一条“错误”日志
    pub fn log_error(&self, content: impl AsRef<str>) {
        self.log(content, LogLevel::Error);
    }

    /// 记录一条日志
    fn log(&self, content: impl AsRef<str>, level: LogLevel) {
        let mut lock = self.inner.lock().unwrap();
        
        for line in content.as_ref().split("\n") {
            println!("{}", line);

            lock.buf.push_back(Line::new(line.to_owned(), level));

            while lock.buf.len() > MAX_LOGS {
                lock.buf.pop_front();
            }
        }
    }
}

pub struct Inner {
    pub buf: LinkedList<Line>,
}

/// 代表单条日志，序列化专用
pub struct LogOutputed {
    /// 日志的产生时间
    pub time: SystemTime,

    /// 日志的内容
    pub content: String,

    /// 日志的重要等级
    pub level: LogLevel,
}

impl Serialize for LogOutputed {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        let unix_ts = self.time.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();

        let mut map = serializer.serialize_map(Some(3))?;
        map.serialize_entry("time", &unix_ts)?;
        map.serialize_entry("content", &self.content)?;
        map.serialize_entry("level", &self.level)?;
        map.end()
    }
}

#[derive(Clone)]
pub struct Line {
    /// 这条日志被阅读过吗
    pub read: bool,

    /// 日志的产生时间
    pub time: SystemTime,

    /// 日志的内容
    pub content: String,

    /// 日志的重要等级
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