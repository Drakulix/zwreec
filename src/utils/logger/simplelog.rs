use std::io::{stderr, stdout, Write};
use log::{LogLevel, LogLevelFilter, LogMetadata, LogRecord, SetLoggerError, set_logger, Log};
use time;
use super::SharedLogger;

pub struct SimpleLogger {
    level: LogLevelFilter,
}

impl SimpleLogger {

    #[allow(dead_code)]
    pub fn init(log_level: LogLevelFilter) -> Result<(), SetLoggerError> {
        set_logger(|max_log_level| {
            max_log_level.set(log_level.clone());
            SimpleLogger::new(log_level)
        })
    }

    #[allow(dead_code)]
    pub fn new(log_level: LogLevelFilter) -> Box<SimpleLogger> {
        Box::new(SimpleLogger { level: log_level })
    }

}

impl Log for SimpleLogger {

    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {

            let stderr = stderr();
            let stdout = stdout();

            let mut stderr_lock = stderr.lock();
            let mut stdout_lock = stdout.lock();

            let cur_time = time::now();

            let _ = match record.level() {
                LogLevel::Error =>
                    writeln!(stderr_lock,
                        "[{}] {}: ({}:{}:{}) - {}",
                            record.level(),
                            record.target(),
                            cur_time.tm_hour,
                            cur_time.tm_min,
                            cur_time.tm_sec,
                            record.args()
                    ),
                LogLevel::Trace =>
                    writeln!(stdout_lock,
                        "[{}] {}: ({}:{}:{}) [{}:{}] - {}",
                            record.level(),
                            record.target(),
                            cur_time.tm_hour,
                            cur_time.tm_min,
                            cur_time.tm_sec,
                            record.location().file(),
                            record.location().line(),
                            record.args()
                    ),
                _ =>
                    writeln!(stdout_lock,
                        "[{}] {}: ({}:{}:{}) - {}",
                            record.level(),
                            record.target(),
                            cur_time.tm_hour,
                            cur_time.tm_min,
                            cur_time.tm_sec,
                            record.args()
                    ),
            };

            stderr_lock.flush().unwrap();
            stdout_lock.flush().unwrap();
        }
    }
}

impl SharedLogger for SimpleLogger {

    fn level(&self) -> LogLevelFilter {
        return self.level;
    }

}

#[cfg(test)]
mod test {
    use std::thread;
    use log::LogLevelFilter;
    use super::*;

    #[test]
    fn test() {
        let _ = SimpleLogger::init(LogLevelFilter::Info);
        info!("Test!");
    }

    //To-Do add a way to check the result automatically
    #[test]
    fn multi_thread_test() {
        let _ = SimpleLogger::init(LogLevelFilter::Info);
        let mut joins = Vec::new();
        for _ in 0..10 {
            joins.push(thread::spawn(move || {
                info!("No corruption should happen in this ultra-extra-super-long string output even if run in multiple threads.");
            }));
        }
        for handle in joins {
            let _ = handle.join();
        }
    }
}
