//! Module providing the FileLogger Implementation

use log::{LogLevel, LogLevelFilter, LogMetadata, LogRecord, SetLoggerError, set_logger, Log};
use time;
use std::fs::File;
use std::io::Write;
use std::sync::Mutex;
use super::SharedLogger;

/// The FileLogger struct. Provides a file based Logger implementation
pub struct FileLogger {
    level: LogLevelFilter,
    file: Mutex<File>,
}

impl FileLogger {

    /// init function. Globally initializes the FileLogger as the one and only used log facility.
    ///
    /// Takes the desired LogLevel and File object (std::fs::File in any write-mode) as argument. They cannot be changed later on.
    /// Fails if another Logger was already initialized.
    ///
    /// # Examples
    /// '''
    /// let _ = FileLogger::init(LogLevelFilter::Info, File::create("my_rust_bin.log").unwrap());
    /// '''
    #[allow(dead_code)]
    pub fn init(log_level: LogLevelFilter, file: File) -> Result<(), SetLoggerError> {
        set_logger(|max_log_level| {
            max_log_level.set(log_level.clone());
            FileLogger::new(log_level, file)
        })
    }

    /// allows to create a new logger, that can be independently used, no matter whats globally set.
    ///
    /// no macros are provided for easy logging in this case and you probably
    /// dont want to use this function, but init().
    ///
    /// Takes the desired LogLevel and File object (std::fs::File in any write-mode) as argument. They cannot be changed later on.
    ///
    /// # Examples
    /// '''
    /// let file_logger = fileLogger::new(LogLevelFilter::Info, File::create("my_rust_bin.log").unwrap());
    /// '''
    #[allow(dead_code)]
    pub fn new(log_level: LogLevelFilter, file: File) -> Box<FileLogger> {
        Box::new(FileLogger { level: log_level, file: Mutex::new(file) })
    }

}

impl Log for FileLogger {

    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {

            let mut file_lock = self.file.lock().unwrap();

            let cur_time = time::now();

            let _ = match record.level() {
                LogLevel::Trace => {
                    writeln!(file_lock,
                        "{:02}:{:02}:{:02} [{}] {}: [{}:{}] {}",
                            record.level(),
                            record.target(),
                            cur_time.tm_hour,
                            cur_time.tm_min,
                            cur_time.tm_sec,
                            record.location().file(),
                            record.location().line(),
                            record.args()
                    ).unwrap();
                },
                _ => {
                    writeln!(file_lock,
                        "{:02}:{:02}:{:02} [{}] {}: {}",
                            record.level(),
                            record.target(),
                            cur_time.tm_hour,
                            cur_time.tm_min,
                            cur_time.tm_sec,
                            record.args()
                    ).unwrap();
                },
            };
        }
    }
}

impl SharedLogger for FileLogger {

    fn level(&self) -> LogLevelFilter {
        self.level
    }

    fn as_log(self: Box<Self>) -> Box<Log> {
        Box::new(*self)
    }

}
