use log::{LogLevelFilter, LogMetadata, LogRecord, SetLoggerError, set_logger, Log};
use super::SharedLogger;

pub struct CombinedLogger {
    level: LogLevelFilter,
    logger: Vec<Box<SharedLogger>>,
}

impl CombinedLogger {

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

            CombinedLogger::new(log_level, logger)
        })
    }

    #[allow(dead_code)]
    pub fn new(log_level: LogLevelFilter, logger: Vec<Box<SharedLogger>>) -> Box<CombinedLogger> {
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
