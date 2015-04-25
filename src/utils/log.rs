extern crate time;

use std::fmt;
use std::io;
use std::io::{Error, Write};
use std::cmp::{Ord, Ordering};

pub enum LogLevel {
    ERROR,
    WARN,
    INFO,
    VERBOSE
}


// ERROR > WARN > INFO > VERBOSE
impl Ord for LogLevel {
    fn cmp(&self, other: &Self) -> Ordering {
        match *self {
            LogLevel::ERROR => match *other {
                        LogLevel::ERROR => Ordering::Equal,
                        _ => Ordering::Greater,
                    },
            LogLevel::WARN => match *other {
                        LogLevel::ERROR => Ordering::Less,
                        LogLevel::WARN => Ordering::Equal,
                        _ => Ordering::Greater,
                    },
            LogLevel::INFO => match *other {
                        LogLevel::VERBOSE => Ordering::Greater,
                        LogLevel::INFO => Ordering::Equal,
                        _ => Ordering::Less,
                    },
            LogLevel::VERBOSE => match *other {
                        LogLevel::VERBOSE => Ordering::Equal,
                        _ => Ordering::Less,
                    },
        }
    }
}

impl Eq for LogLevel {}

impl PartialEq for LogLevel {

    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }

}

impl PartialOrd for LogLevel {

    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }

    fn lt(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Less
    }

    fn le(&self, other: &Self) -> bool {
        self.cmp(other) != Ordering::Greater
    }

    fn gt(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Greater
    }

    fn ge(&self, other: &Self) -> bool {
        self.cmp(other) != Ordering::Less
    }
}

/// Set global log level here
static LOG_LEVEL: LogLevel = LogLevel::INFO;

/// the log function returning a result
///
/// # Examples
/// '''
/// log(LogLevel::ERROR, format_args!("This is bad!")).unwrap();
/// '''
pub fn log(level: LogLevel, args: fmt::Arguments) -> Result<(), Error> {

    if level < LOG_LEVEL {
        return Ok(());
    }

    let stderr = io::stderr();
    let stdout = io::stdout();

    let mut stderr_lock = stderr.lock();
    let mut stdout_lock = stdout.lock();

    let cur_time = time::now();

    match level {
        LogLevel::ERROR => writeln!(stderr_lock, "[ERROR] {}:{}:{} - {}", cur_time.tm_hour, cur_time.tm_min, cur_time.tm_sec, args),
        LogLevel::WARN => writeln!(stdout_lock, "[WARN] {}:{}:{} - {}", cur_time.tm_hour, cur_time.tm_min, cur_time.tm_sec, args),
        LogLevel::INFO => writeln!(stdout_lock, "[INFO] {}:{}:{} - {}", cur_time.tm_hour, cur_time.tm_min, cur_time.tm_sec, args),
        LogLevel::VERBOSE => writeln!(stdout_lock, "[VERBOSE] {}:{}:{} - {}", cur_time.tm_hour, cur_time.tm_min, cur_time.tm_sec, args),
    }

}

/// a log function ignoring any failure
/// "log something, if possible"
///
/// # Examples
/// '''
/// try_log(LogLevel::ERROR, format_args!("This is bad!"));
/// '''
pub fn try_log(level: LogLevel, args: fmt::Arguments) {
    let _ = log(level, args);
}

// Helper Macro

/// Error Macro
///
/// # Examples
/// '''
/// error!("This is bad!");
/// '''
macro_rules! log_error {
    ($($arg:tt)*) => (::utils::log::try_log(::utils::log::LogLevel::ERROR, format_args!($($arg)*)));
}

/// Warning Macro
///
/// # Examples
/// '''
/// log_warn!("This is maybe bad?!");
/// '''
macro_rules! log_warn {
    ($($arg:tt)*) => (::utils::log::try_log(::utils::log::LogLevel::WARN, format_args!($($arg)*)));
}

/// Info Macro
///
/// # Examples
/// '''
/// log_info!("This is just to let you know!");
/// '''
macro_rules! log_info {
    ($($arg:tt)*) => (::utils::log::try_log(::utils::log::LogLevel::INFO, format_args!($($arg)*)));
}

/// Verbose Macro
///
/// # Examples
/// '''
/// log_verbose!("You must really like spam!");
/// '''
macro_rules! log_verbose {
    ($($arg:tt)*) => (::utils::log::try_log(::utils::log::LogLevel::VERBOSE, format_args!($($arg)*)));
}


#[test]
fn test() {
    log_info!("Test!");
}

//To-Do add a way to check the result automatically
#[test]
fn multi_thread_test() {
    let mut joins = Vec::new();
    for _ in 0..10 {
        joins.push(std::thread::spawn(move || {
            log_info!("No corruption should happen in this ultra-extra-super-long string output even if run in multiple threads.");
        }));
    }
    for handle in joins {
        let _ = handle.join();
    }
}
