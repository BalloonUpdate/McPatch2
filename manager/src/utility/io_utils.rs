use std::io::Read;

/// 从`read`里不断读取内容，直到末尾。
/// 
/// 每当遇到`\n`字符时，调用一次`f`
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