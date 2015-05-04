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

pub trait SharedLogger: Log {
    fn level(&self) -> LogLevelFilter;
    
    fn as_log(self: Box<Self>) -> Box<Log>;
}
