//! Module providing the TermLogger Implementation

use log::{LogLevel, LogLevelFilter, LogMetadata, LogRecord, SetLoggerError, set_logger, Log};
use time;
use term;
use term::{StderrTerminal, StdoutTerminal, color};
use std::sync::Mutex;
use super::SharedLogger;

/// The TermLogger struct. Provides a stderr/stdout based Logger implementation
pub struct TermLogger {
    level: LogLevelFilter,
    stderr: Mutex<Box<StderrTerminal>>,
    stdout: Mutex<Box<StdoutTerminal>>,
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
        Box::new(TermLogger { level: log_level, stderr: Mutex::new(term::stderr().unwrap()), stdout: Mutex::new(term::stdout().unwrap()) })
    }

}

impl Log for TermLogger {

    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {

            let mut stderr_lock = self.stderr.lock().unwrap();
            let mut stdout_lock = self.stdout.lock().unwrap();

            let cur_time = time::now();

            let _ = match record.level() {
                LogLevel::Error => {
                    write!(stderr_lock, "[").unwrap();
                    stderr_lock.fg(color::RED).unwrap();
                    write!(stderr_lock, "{}", record.level()).unwrap();
                    let _ = stderr_lock.reset();
                    writeln!(stderr_lock,
                        "] {}: ({}:{}:{}) - {}",
                            record.target(),
                            cur_time.tm_hour,
                            cur_time.tm_min,
                            cur_time.tm_sec,
                            record.args()
                    ).unwrap();
                },
                LogLevel::Warn => {
                    write!(stdout_lock, "[").unwrap();
                    stdout_lock.fg(color::YELLOW).unwrap();
                    write!(stdout_lock, "{}", record.level()).unwrap();
                    let _ = stdout_lock.reset();
                    writeln!(stdout_lock,
                        "] {}: ({}:{}:{}) - {}",
                            record.target(),
                            cur_time.tm_hour,
                            cur_time.tm_min,
                            cur_time.tm_sec,
                            record.args()
                    ).unwrap();
                },
                LogLevel::Info => {
                    write!(stdout_lock, "[").unwrap();
                    stdout_lock.fg(color::BLUE).unwrap();
                    write!(stdout_lock, "{}", record.level()).unwrap();
                    let _ = stdout_lock.reset();
                    writeln!(stdout_lock,
                        "] {}: ({}:{}:{}) - {}",
                            record.target(),
                            cur_time.tm_hour,
                            cur_time.tm_min,
                            cur_time.tm_sec,
                            record.args()
                    ).unwrap();
                },
                LogLevel::Debug => {
                    write!(stdout_lock, "[").unwrap();
                    stdout_lock.fg(color::CYAN).unwrap();
                    write!(stdout_lock, "{}", record.level()).unwrap();
                    let _ = stdout_lock.reset();
                    writeln!(stdout_lock,
                        "] {}: ({}:{}:{}) - {}",
                            record.target(),
                            cur_time.tm_hour,
                            cur_time.tm_min,
                            cur_time.tm_sec,
                            record.args()
                    ).unwrap();
                },
                LogLevel::Trace => {
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
                    ).unwrap();
                },
            };

            stderr_lock.flush().unwrap();
            stdout_lock.flush().unwrap();
        }
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
