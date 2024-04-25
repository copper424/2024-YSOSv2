use crate::*;
use alloc::string::{String, ToString};
use alloc::vec;

pub struct Stdin;
pub struct Stdout;
pub struct Stderr;

impl Stdin {
    fn new() -> Self {
        Self
    }

    pub fn read_line(&self) -> String {
        // FIXME: allocate string
        let mut buf = String::new();
        // FIXME: read from input buffer
        //       - maybe char by char?
        let mut char_buf = [0u8; 4];
        while let Some(len) = sys_read(0, &mut char_buf) {
            if len == 4 {
                let ch = core::str::from_utf8(&mut char_buf)
                    .expect("failed to convert the u8 array into a str")
                    .chars()
                    .next()
                    .unwrap();
                // FIXME: handle backspace / enter...
                match ch {
                    '\n' | '\r' => {
                        sys_write(1, &char_buf);
                        break;
                    }
                    '\x04' => {
                        buf.clear();
                        buf.push(ch);
                        break;
                    }
                    '\x08' | '\x7F' => {
                        if !buf.is_empty() {
                            io::print!("\x08\x20\x08");
                            buf.pop();
                        }
                    }
                    // ignore other control character
                    '\x00'..='\x1F' => {}
                    c => {
                        buf.push(ch);
                        // echo the input character
                        io::print!("{}", c);
                    }
                }
            } else {
                // len == 0
                continue;
            }
        }
        // FIXME: return string
        buf
    }
}

impl Stdout {
    fn new() -> Self {
        Self
    }

    pub fn write(&self, s: &str) {
        sys_write(1, s.as_bytes());
    }
}

impl Stderr {
    fn new() -> Self {
        Self
    }

    pub fn write(&self, s: &str) {
        sys_write(2, s.as_bytes());
    }
}

pub fn stdin() -> Stdin {
    Stdin::new()
}

pub fn stdout() -> Stdout {
    Stdout::new()
}

pub fn stderr() -> Stderr {
    Stderr::new()
}
