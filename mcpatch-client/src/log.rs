use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::RwLock;
use std::time::SystemTime;

static LOG_HANDLERS: RwLock<Vec<Box<dyn MessageHandler + Send>>> = RwLock::new(Vec::new());

static LOG_PREFIX: RwLock<String> = RwLock::new(String::new());

pub trait MessageHandler : Sync {
    fn record(&self, message: &Message);
}

#[derive(PartialEq, PartialOrd)]
pub enum MessageLevel {
    All,
    Debug,
    Info,
    Warning,
    Error,
    None
}

pub struct Message<'a> {
    pub time: SystemTime,
    pub level: MessageLevel,
    pub prefix: &'a str,
    pub content: &'a str,
    pub new_line: bool,
}

pub fn set_log_prefix(prefix: &str) {
    *LOG_PREFIX.write().unwrap() = prefix.to_owned();
}

pub fn add_log_handler(handler: Box<dyn MessageHandler + Send>) {
    LOG_HANDLERS.write().unwrap().push(handler);
}

pub fn log_error(content: impl AsRef<str>) {
    log_message(content, MessageLevel::Error, true);
}

pub fn log_info(content: impl AsRef<str>) {
    log_message(content, MessageLevel::Info, true);
}

pub fn log_debug(content: impl AsRef<str>) {
    log_message(content, MessageLevel::Debug, true);
}

pub fn log_info_s(content: impl AsRef<str>) {
    log_message(content, MessageLevel::Info, false);
}

fn log_message(content: impl AsRef<str>, level: MessageLevel, new_line: bool) {
    let prefix = LOG_PREFIX.read().unwrap();

    let msg = Message {
        time: SystemTime::now(),
        level,
        prefix: &prefix,
        content: content.as_ref(),
        new_line,
    };

    for handler in LOG_HANDLERS.read().unwrap().iter() {
        handler.record(&msg)
    }
}

pub struct ConsoleHandler {
    filter: MessageLevel
}

impl ConsoleHandler {
    pub fn new(filter: MessageLevel) -> Self {
        Self { filter }
    }
}

impl MessageHandler for ConsoleHandler {
    fn record(&self, message: &Message) {
        if message.level < self.filter {
            return;
        }

        let mut buf = String::with_capacity(message.content.len() + 128);

        fn push_prefix(buf: &mut String, tag: &str) {
            if !tag.is_empty() {
                buf.push('[');
                buf.push_str(tag);
                buf.push(']');
            };
        }

        push_prefix(&mut buf, &message.prefix);

        for c in message.content.chars() {
            if c == '\n' {
                push_prefix(&mut buf, &message.prefix);
            }

            buf.push(c);
        }

        if message.new_line {
            println!("{}", buf);
        } else {
            print!("{}", buf);
        }
    }
}

pub struct FileHandler {
    file: Mutex<std::fs::File>,
}

impl FileHandler {
    pub fn new(log_file: &PathBuf) -> Self {
        Self {
            file: Mutex::new(std::fs::File::options()
                .create(true)
                .write(true)
                .truncate(true)
                .open(log_file).unwrap())
        }
    }
}

impl MessageHandler for FileHandler {
    fn record(&self, message: &Message) {
        let mut buf = String::with_capacity(message.content.len() + 128);

        fn push_prefix(buf: &mut String, tag: &str) {
            if !tag.is_empty() {
                buf.push('[');
                buf.push_str(tag);
                buf.push(']');
            };
        }

        push_prefix(&mut buf, &message.prefix);

        for c in message.content.chars() {
            if c == '\n' {
                push_prefix(&mut buf, &message.prefix);
            }

            buf.push(c);
        }

        if message.new_line {
            buf.push('\n');
        }

        self.file.lock().unwrap().write_all(buf.as_bytes()).unwrap();
    }
}