//!
//! The logger module provides various Log-Implementations to setup a Logging facility
//!
//! It provides the following Logger implementations:
//! - SimpleLogger (uses printfn!{})
//! - TermLogger (logs directly to stderr, color support)
//! - FileLogger (logs to a log file)
//! - CombinedLogger (allows to form combinations of the above loggers)
//!
//! Only one Logger should be initialized of the start of your program
//! through the Logger::init(...) method. For the actual calling syntax
//! take a look at the documentation of a specific implementation.
//!

pub mod termlog;
pub mod filelog;
pub mod simplelog;
pub mod comblog;

pub use self::termlog::TermLogger;
pub use self::filelog::FileLogger;
pub use self::simplelog::SimpleLogger;
pub use self::comblog::CombinedLogger;
pub use log::LogLevelFilter;

use log::Log;

/// Trait to have a common interface to obtain the LogLevel of Loggers
///
/// Necessary for CombinedLogger to calculate
/// the lowest used LogLevel.
///
pub trait SharedLogger: Log {
    /// Returns the set LogLevel for this Logger
    ///
    /// # Examples
    ///
    /// '''
    /// let logger = SimpleLogger::new(LogLevelFilter::Info);
    /// println!("{}", logger.level());
    /// '''
    fn level(&self) -> LogLevelFilter;
    
    fn as_log(self: Box<Self>) -> Box<Log>;
}
