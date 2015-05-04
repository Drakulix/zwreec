//! Module providing the CombinedLogger Implementation

use log::{LogLevelFilter, LogMetadata, LogRecord, SetLoggerError, set_logger, Log};
use super::SharedLogger;

/// The CombinedLogger struct. Provides a Logger implementation that proxies multiple Loggers as one/
///
/// The purpose is to allow all Logger to be set globally and therefor used at one
pub struct CombinedLogger {
    level: LogLevelFilter,
    logger: Vec<Box<Log>>,
}

impl CombinedLogger {

    /// init function. Globally initializes the CombinedLogger as the one and only used log facility.
    ///
    /// Takes all used Loggers as a Vec Argument. None of those Loggers should already be set globally.
    /// All Loggers need to implement log::Log and logger::SharedLogger and need to provide a way to be
    /// initialized without calling set_logger. All loggers of the logger module provide a new(...) method
    /// for that purpose.
    /// Fails if another logger is already set globally.
    ///
    /// # Examples
    /// '''
    /// let _ = CombinedLogger::init(
    ///             vec![
    ///                 TermLogger::new(LogLevelFilter::Info),
    ///                 FileLogger::new(LogLevelFilter::Info, File::create("my_rust_bin").unwrap())
    ///             ]
    ///         );
    /// '''
    #[allow(dead_code)]
    pub fn init(logger: Vec<Box<SharedLogger>>) -> Result<(), SetLoggerError> {
        set_logger(|max_log_level| {

            let mut log_level = LogLevelFilter::Off;
            for log in &logger {
                if log_level < log.level() {
                    log_level = log.level();
                }
            }
            max_log_level.set(log_level.clone());

            CombinedLogger::new(log_level, logger.into_iter().map(|logger| logger.as_log()).collect())
        })
    }

    /// allows to create a new logger, that can be independently used, no matter whats globally set.
    ///
    /// no macros are provided for easy logging in this case and you probably
    /// dont want to use this function, but init().
    ///
    /// Takes all used Loggers as a Vec Argument and the lowest LogLevel of all set loggers
    /// Take LogLevelFilter::Trace, if you have no way to check. This might cause overhead.
    /// All Loggers need to implement log::Log.
    ///
    /// # Examples
    /// '''
    /// let combined_logger = CombinedLogger::new(
    ///             LogLevelFilter::Debug,
    ///             vec![
    ///                 TermLogger::new(LogLevelFilter::Debug),
    ///                 FileLogger::new(LogLevelFilter::Info, File::create("my_rust_bin").unwrap())
    ///             ]
    ///         );
    /// '''
    #[allow(dead_code)]
    pub fn new(log_level: LogLevelFilter, logger: Vec<Box<Log>>) -> Box<CombinedLogger> {
        Box::new(CombinedLogger { level: log_level, logger: logger })
    }

}

impl Log for CombinedLogger {

    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {
            for log in &self.logger {
                log.log(record);
            }
        }
    }
}
