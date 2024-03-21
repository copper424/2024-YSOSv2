use log::{Level, Metadata, Record};

pub fn init(boot_info: &'static boot::BootInfo) {
    static LOGGER: Logger = Logger;
    log::set_logger(&LOGGER).unwrap();

    // FIXME: Configure the logger
    log::set_max_level(boot_info.log_level.to_level_filter());
    info!("Logger Initialized.");
}

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        // FIXME: Implement the logger with serial output
        if self.enabled(record.metadata()) {
            match record.level() {
                Level::Trace => {
                    println!(
                        "\x1b[31m[{}]: {}@{}: {}\x1b[0m",
                        record.level(),
                        record.file_static().unwrap(),
                        record.line().unwrap(),
                        record.args()
                    );
                }
                Level::Error => {
                    println!(
                        "\x1b[32m[{}]: {}@{}: {}\x1b[0m",
                        record.level(),
                        record.file_static().unwrap(),
                        record.line().unwrap(),
                        record.args()
                    );
                }
                Level::Warn => {
                    println!(
                        "\x1b[33m[{}]: {}@{}: {}\x1b[0m",
                        record.level(),
                        record.file_static().unwrap(),
                        record.line().unwrap(),
                        record.args()
                    );
                }
                Level::Info => {
                    println!(
                        "\x1b[34m[{}]: {}@{}: {}\x1b[0m",
                        record.level(),
                        record.file_static().unwrap(),
                        record.line().unwrap(),
                        record.args()
                    );
                }
                Level::Debug => {
                    println!(
                        "\x1b[35m[{}]: {}@{}: {}\x1b[0m",
                        record.level(),
                        record.file_static().unwrap(),
                        record.line().unwrap(),
                        record.args()
                    );
                }
            }
        }
    }

    fn flush(&self) {}
}
