pub mod ext;
pub mod tc;
pub mod counted_write;
pub mod limited_read;

use std::io::Read;

pub fn read_into_lines<R>(mut read: impl Read, mut f: impl FnMut(&str) -> R) {
    let mut line = Vec::with_capacity(128);
    let mut buf = [0u8; 4 * 1024];

    loop {
        let count = read.read(&mut buf).unwrap();

        if count == 0 {
            if !line.is_empty() {
                let l = std::str::from_utf8(&line).unwrap().trim();
                
                if !l.is_empty() {
                    f(l);
                    line.clear();
                }
            }

            break;
        }

        for b in &buf[0..count] {
            let b = *b;
            
            if b == '\n' as u8 {
                if !line.is_empty() {
                    let l = std::str::from_utf8(&line).unwrap().trim();

                    if !l.is_empty() {
                        f(l);
                        line.clear();
                    }
                }
            } else {
                line.push(b);
            }
        }
    }
}

pub fn join_string(iter: impl Iterator<Item = impl AsRef<str>>, split: &str) -> String {
    let mut result = String::new();
    let mut insert = false;

    for e in iter {
        if insert {
            result.push_str(split);
        }

        result.push_str(e.as_ref());

        insert = true;
    }

    result
}

pub fn is_running_under_cargo() -> bool {
    std::env::vars().any(|p| p.0.eq_ignore_ascii_case("CARGO"))
}
