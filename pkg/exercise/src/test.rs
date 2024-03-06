mod program1 {
    pub fn count_down(seconds: u64) {
        for i in 1..=seconds {
            println!("There is {} second left", i);
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
        println!("Countdown finished.");
    }
    pub fn read_and_print(file_path: &str) {
        let mut file_handle = std::fs::File::open(file_path).expect("File not found!");
        let mut file_str = String::new();
        let _ = std::io::Read::read_to_string(&mut file_handle, &mut file_str);
        println!("{}", file_str);
    }
    pub fn file_size(file_path: &str) -> Result<u64, &str> {
        let mut file_handle = std::fs::File::open(file_path).map_err(|_e| {
            println!("Inner error is {}", _e);
            "File not found!"
        })?;
        let metadata = file_handle
            .metadata()
            .map_err(|_e| "Cannot parse the metadata of the file.")?;
        Ok(metadata.len())
    }
}
mod program2 {
    pub fn humanized_size(size: u64) -> (f64, &'static str) {
        static UNIT: [&str; 4] = ["B", "KiB", "MiB", "GiB"];
        let mut size = size as f64;
        let mut idx = 0;
        while size >= 1024f64 {
            size /= 1024f64;
            idx += 1;
        }
        (size, UNIT[idx])
    }
}
mod program3 {
    pub fn print_with_color() {
        println!("\x1b[32mINFO: \x1b[0mHello, world!");
        println!("\x1b[33m\x1b[1m\x1b[4mWARNING\x1b[24m: I'm a teapot!\x1b[0m");
        println!(
            "\x1b[31m\x1b[1m{:^50}\x1b[0m",
            format!("ERROR: KERNEL PANIC!!!")
        );
    }
}
mod program4 {
    pub enum Shape {
        Rectangle { width: f64, height: f64 },
        Circle { radius: f64 },
    }
    impl Shape {
        pub fn area(&self) -> f64 {
            match self {
                Shape::Rectangle { width, height } => {
                    return width * height;
                }
                Shape::Circle { radius } => {
                    return std::f64::consts::PI * radius * radius;
                }
            }
        }
    }
}
mod program5 {
    use std::sync::atomic::{AtomicU16, Ordering};
    #[derive(Eq, PartialEq, PartialOrd, Ord, Debug)]
    pub struct UniqueId(u16);
    impl UniqueId {
        pub fn new() -> Self {
            static mut count: AtomicU16 = AtomicU16::new(0);
            unsafe {
                return Self(count.fetch_add(1, Ordering::SeqCst));
            }
        }
    }
}
mod program6 {
    use std::{env::current_dir, fs, io};

    fn read_and_print(file_path: &std::path::Path) -> Result<(), std::io::Error> {
        let mut file_handle = std::fs::File::open(file_path)?;
        let mut file_str = String::new();
        std::io::Read::read_to_string(&mut file_handle, &mut file_str)?;
        println!("{}", file_str);
        Ok(())
    }
    pub fn file_size(file_path: &str) -> Result<u64, &str> {
        let mut file_handle = std::fs::File::open(file_path).map_err(|_e| {
            println!("Inner error is {}", _e);
            "File not found!"
        })?;
        let metadata = file_handle
            .metadata()
            .map_err(|_e| "Cannot parse the metadata of the file.")?;
        Ok(metadata.len())
    }
    pub fn main() {
        use std::path::{Path, PathBuf};
        let mut current_dir = std::env::current_dir().unwrap();
        loop {
            println!("current directory is: {}", current_dir.display());

            let mut tmp = Default::default();
            io::stdin().read_line(&mut tmp).unwrap();
            let mut args = tmp.split_whitespace();
            let command = args.next().unwrap();

            match command {
                "cd" => {
                    let new_dir = args.next().unwrap_or(".");
                    let new_dir = Path::new(new_dir);
                    let new_dir = current_dir.join(new_dir);
                    current_dir = new_dir.canonicalize().unwrap();
                }
                "ls" => {
                    for entry in fs::read_dir(&current_dir).unwrap() {
                        let entry = entry.unwrap();
                        let path = entry.path();

                        // 获取文件信息
                        let metadata = fs::metadata(&path).unwrap();
                        // 输出文件信息
                        println!(
                            "{:10} {:?} {}",
                            metadata.len(),
                            metadata.modified().unwrap(),
                            path.display()
                        );
                    }
                }
                "cat" => {
                    let filename = args.next().unwrap();
                    let path = Path::new(filename);
                    println!("Read and print result is {:#?}",read_and_print(path));
                }

                // 退出
                "exit" => break,

                // 未知命令
                _ => println!("unknown command: {}", command),
            }
        }
    }
}
#[cfg(test)]
mod test {
    use core::time;

    use log::{debug, error, info, trace, warn};

    use super::program5::UniqueId;

    #[test]
    fn main() {
        use super::program1::*;
        count_down(5);
        read_and_print("C:\\Windows\\System32\\drivers\\etc\\hosts");
        for _ in 0..2 {
            let mut tmp = String::new();
            println!("input a file path");
            match std::io::stdin().read_line(&mut tmp) {
                Ok(_) => {
                    println!("The path is {}", tmp);
                    println!("read succeeded!")
                }
                Err(e) => {
                    println!("{}", e);
                    continue;
                }
            }
            // drop the '\n' character
            tmp.pop();
            match file_size(&tmp) {
                Ok(res) => {
                    println!(
                        "successfully opened the file and its size is {} byte(s)",
                        res
                    )
                }
                Err(e) => {
                    println!("failed to open the file.The error is {}", e);
                }
            }
        }
    }
    #[test]
    fn test_humanized_size() {
        let byte_size = 1554056;
        let (size, unit) = super::program2::humanized_size(byte_size);
        assert_eq!(
            "Size :  1.4821 MiB",
            format!("Size :  {:.4} {}", size, unit)
        );
    }
    #[test]
    fn test_colored_output() {
        use super::program3::print_with_color;
        print_with_color();
    }
    #[test]
    fn test_area() {
        use super::program4::*;
        let rectangle = Shape::Rectangle {
            width: 10.0,
            height: 20.0,
        };
        let circle = Shape::Circle { radius: 10.0 };

        assert_eq!(rectangle.area(), 200.0);
        assert_eq!(circle.area(), 314.1592653589793);
    }
    #[test]
    fn test_unique_id() {
        use super::program5::UniqueId;
        let id1 = UniqueId::new();
        let id2 = UniqueId::new();
        assert_ne!(id1, id2);
    }
    #[test]
    fn test_log_with_color() {
        use log::{Level, LevelFilter, Metadata, Record};

        struct SimpleLogger;

        impl log::Log for SimpleLogger {
            fn enabled(&self, metadata: &Metadata) -> bool {
                metadata.level() <= Level::Trace
            }

            fn log(&self, record: &Record) {
                if self.enabled(record.metadata()) {
                    // println!("{} - {}", record.level(), record.args());
                    match record.level() {
                        Level::Warn => {
                            println!("\x1b[31m{} - {}\x1b[0m", Level::Warn, record.args());
                        }
                        Level::Info => {
                            println!("\x1b[32m{} - {}\x1b[0m", Level::Info, record.args());
                        }
                        Level::Error => {
                            println!("\x1b[33m{} - {}\x1b[0m", Level::Error, record.args());
                        }
                        Level::Debug => {
                            println!("\x1b[34m{} - {}\x1b[0m", Level::Debug, record.args());
                        }
                        Level::Trace => {
                            println!("\x1b[35m{} - {}\x1b[0m", Level::Trace, record.args());
                        }
                    }
                }
            }

            fn flush(&self) {}
        }
        static MY_LOGGER: SimpleLogger = SimpleLogger;
        log::set_logger(&MY_LOGGER).unwrap();
        log::set_max_level(LevelFilter::Trace);
        warn!("This is a warning log");
        info!("This is a information log");
        error!("This is a error log");
        debug!("This is a debug log");
        trace!("This is a trace log");
    }
    #[test]
    fn test_simple_shell() {
        use super::program6::main;
        main();
    }
    #[test]
    fn test_unique_id2() {
        #[derive(Debug, Eq, PartialEq, PartialOrd, Ord)]
        struct UniqueId(u16);
        impl UniqueId {
            fn new() -> Self {
                static mut count: u16 = 0;
                unsafe {
                    count += 1;
                    Self(count)
                }
            }
        }
        for _ in 0..10000 {
            let handle0 = std::thread::spawn(|| super::program5::UniqueId::new());
            let handle1 = std::thread::spawn(|| super::program5::UniqueId::new());
            let id0 = handle0.join().unwrap();
            let id1 = handle1.join().unwrap();
            assert_ne!(id0, id1);
        }
        println!("Passed test 1");
        for _ in 0..10000 {
            let handle2 = std::thread::spawn(|| UniqueId::new());
            let handle3 = std::thread::spawn(|| UniqueId::new());
            let id2 = handle2.join().unwrap();
            let id3 = handle3.join().unwrap();
            assert_ne!(id2, id3);
        }
        println!("Passed test 2");
    }
}
