use alloc::string::String;
use crossbeam_queue::ArrayQueue;

use crate::serial::get_serial_for_sure;
type Key = char;

const BUFFER_SIZE: usize = 128;

lazy_static! {
    static ref INPUT_BUF: ArrayQueue<Key> = ArrayQueue::new(BUFFER_SIZE);
}

#[inline]
pub fn push_key(key: Key) {
    if INPUT_BUF.push(key).is_err() {
        warn!("Input buffer is full. Dropping key '{:?}'", key);
    }
}

#[inline]
pub fn try_get_key() -> Option<Key> {
    INPUT_BUF.pop()
}

pub fn pop_key() -> Key {
    loop {
        if let Some(val) = try_get_key() {
            return val;
        }
    }
}

pub fn get_line() -> String {
    let mut line = String::with_capacity(BUFFER_SIZE);
    loop {
        let val = pop_key();
        if val == '\n' || val == '\r' {
            break;
        }
        if (val == '\x08' || val == '\x7F') && !line.is_empty() {
            get_serial_for_sure().backspace();
            line.pop();
        } else {
            print!("{}", val);
            line.push(val);
        }
    }
    line
}
