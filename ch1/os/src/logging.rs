use log::{Record, Level, Metadata, Log, LevelFilter};

struct SimpleLogger;

impl Log for SimpleLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let color = match record.level() {
                Level::Error => 31,
                Level::Warn => 93,
                Level::Info => 34,
                Level::Debug => 32,
                Level::Trace => 90,
            };
            // Ref: <https://docs.rs/log/0.4.19/log/struct.Record.html>
            println!("\u{1B}[{}m[{:5>}] {}\u{1B}[0m",
                color,
                record.level(), /* The verbosity level of the message */
                record.args() /* The message body */
            );
        }
    }
    fn flush(&self) {}
}

pub fn init() {
    static LOGGER: SimpleLogger = SimpleLogger;
    let _ = log::set_logger(&LOGGER).map(|()| {
        log::set_max_level(match option_env!("LOG") {
            Some(level) => {
                match level {
                    "ERROR" => LevelFilter::Error,
                    "WARN" => LevelFilter::Warn,
                    "INFO" => LevelFilter::Info,
                    "DEBUG" => LevelFilter::Debug,
                    "TRACE" => LevelFilter::Trace,
                    _ => LevelFilter::Off,
                }
            },
            None => LevelFilter::Off,
        });
    });
}