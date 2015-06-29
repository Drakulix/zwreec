//! Module providing the TermLogger Implementation

use log::{LogLevel, LogLevelFilter, LogMetadata, LogRecord, SetLoggerError, set_logger, Log};
use time;
use term;
use term::{StderrTerminal, color};
use std::sync::Mutex;
use std::io::Error;
use super::SharedLogger;

/// The TermLogger struct. Provides a stderr based Logger implementation
pub struct TermLogger {
    level: LogLevelFilter,
    stderr: Mutex<Box<StderrTerminal>>,
}

impl TermLogger {

    /// init function. Globally initializes the TermLogger as the one and only used log facility.
    ///
    /// Takes the desired LogLevel as argument. It cannot be changed later on.
    /// Fails if another Logger was already initialized.
    ///
    /// # Examples
    /// '''
    /// let _ = TermLogger::init(LogLevelFilter::Info);
    /// '''
    #[allow(dead_code)]
    pub fn init(log_level: LogLevelFilter) -> Result<(), SetLoggerError> {
        set_logger(|max_log_level| {
            max_log_level.set(log_level.clone());
            TermLogger::new(log_level)
        })
    }

    /// allows to create a new logger, that can be independently used, no matter whats globally set.
    ///
    /// no macros are provided for easy logging in this case and you probably
    /// dont want to use this function, but init().
    ///
    /// Takes the desired LogLevel as argument. It cannot be changed later on.
    ///
    /// # Examples
    /// '''
    /// let term_logger = TermLogger::new(LogLevelFilter::Info);
    /// '''
    #[allow(dead_code)]
    pub fn new(log_level: LogLevelFilter) -> Box<TermLogger> {
        Box::new(TermLogger { level: log_level, stderr: Mutex::new(term::stderr().unwrap()) })
    }

    fn try_log(&self, record: &LogRecord) -> Result<(), Error> {

        if self.enabled(record.metadata()) {
            let mut stderr_lock = self.stderr.lock().unwrap();

            let cur_time = time::now();

            let color = match record.level() {
                LogLevel::Error => color::RED,
                LogLevel::Warn => color::YELLOW,
                LogLevel::Info => color::BLUE,
                LogLevel::Debug => color::CYAN,
                LogLevel::Trace => color::WHITE
            };

            if self.level() <= LogLevel::Warn {
                try!(write!(stderr_lock, "["));
                try!(stderr_lock.fg(color));
                try!(write!(stderr_lock, "{}", record.level()));
                try!(stderr_lock.reset());
                try!(writeln!(stderr_lock,
                    "] {}",
                        record.args()
                ));
            } else {
                try!(write!(stderr_lock, "{:02}:{:02}:{:02} [",
                            cur_time.tm_hour,
                            cur_time.tm_min,
                            cur_time.tm_sec));

                match record.level() {
                    LogLevel::Error |
                    LogLevel::Warn  |
                    LogLevel::Info  |
                    LogLevel::Debug => {
                        try!(stderr_lock.fg(color));
                        try!(write!(stderr_lock, "{}", record.level()));
                        try!(stderr_lock.reset());
                        try!(writeln!(stderr_lock,
                            "] {}: {}",
                                record.target(),
                                record.args()
                        ));
                    },
                    LogLevel::Trace => {
                        try!(writeln!(stderr_lock,
                            "[{}] {}: [{}:{}] - {}",
                                record.level(),
                                record.target(),
                                record.location().file(),
                                record.location().line(),
                                record.args()
                        ));
                    },
                };
            }

            try!(stderr_lock.flush());
        };

        Ok(())
    }
}

impl Log for TermLogger {

    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &LogRecord) {
        let _ = self.try_log(record);
    }
}

impl SharedLogger for TermLogger {

    fn level(&self) -> LogLevelFilter {
        self.level
    }

    fn as_log(self: Box<Self>) -> Box<Log> {
        Box::new(*self)
    }

}
