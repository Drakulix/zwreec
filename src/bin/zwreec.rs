extern crate zwreec;
extern crate getopts;
#[macro_use] extern crate log;
extern crate time;
extern crate term;

use std::env;
use std::vec::Vec;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::exit;

use zwreec::config;

mod logger;

// found in:
// http://stackoverflow.com/a/27590832
macro_rules! print_stderr(
    ($($arg:tt)*) => (
        match write!(&mut ::std::io::stderr(), $($arg)* ) {
            Ok(_) => {},
            Err(x) => panic!("Unable to write to stderr: {}", x),
        }
    )
);

/// Prints usage information
/// The verbose flag signals if all options should be shown.
/// NOTE: This is similar to librustc_driver's usage function.
fn usage(verbose: bool) {
    let options = if verbose {
        config::zwreec_options()
    } else {
        config::zwreec_short_options()
    };

    let brief = format!("Usage: zwreec [-hV] [-vq] [-l [LOGFILE]] [-o OUTPUT] INPUT");

    println!("{}\n
Additional help:
    --help -v           Print the full set of options zwreec accepts", 
        options.usage(&brief));
}

/// NOTE: This is similar to librustc_driver's handle_options function.
fn parse_arguments(args: Vec<String>) -> getopts::Matches {
    let mut loggers: Vec<Box<logger::SharedLogger>> = vec![];

    if args.is_empty() {
        // No options provided, print usage and exit
        usage(false);
        exit(1);
    }

    let opts = config::zwreec_options();

    let matches = match opts.parse(&args[1..]) {
        Ok(m)  => m,
        Err(f) => {
            // parsing error
            // display usage and return
            print_stderr!("{}\n", f.to_string());
            exit(1);
        }
    };

    if matches.opt_present("help") {
        usage(matches.opt_present("verbose"));
        exit(1);
    }

    if matches.opt_present("version") {
        println!("zwreec {}", match option_env!("CFG_VERSION") {
            Some(s) => s.to_string(),
            None => format!("{}.{}.{}{}",
                            env!("CARGO_PKG_VERSION_MAJOR"),
                            env!("CARGO_PKG_VERSION_MINOR"),
                            env!("CARGO_PKG_VERSION_PATCH"),
                            option_env!("CARGO_PKG_VERSION_PRE").unwrap_or(""))
        });
        exit(0);
    }

    if matches.opt_present("verbose") {
        // set log level to verbose
        loggers.push(logger::TermLogger::new(
                match matches.opt_count("v") {
                    1 => logger::LogLevelFilter::Info,
                    2 => logger::LogLevelFilter::Debug,
                    _ => logger::LogLevelFilter::Trace,
                }));
    } else if matches.opt_present("quiet") {
        // set log level to error
        loggers.push(logger::TermLogger::new(logger::LogLevelFilter::Error));
    } else {
        // set log level to warn
        loggers.push(logger::TermLogger::new(logger::LogLevelFilter::Warn));
    }

    if matches.opt_present("logfile") {
        // sets a logger to output to logfile
        let name = if let Some(n) = matches.opt_str("logfile") {
            n
        } else {
            "zwreec.log".to_string()
        };
        loggers.push(logger::FileLogger::new(
                        logger::LogLevelFilter::Trace,
                        File::create(name).unwrap())
            );
    }

    // TODO: This might not belong in a function called parse_arguments
    // activate logger
    let _ = logger::CombinedLogger::init(loggers);

    matches
}

fn main() {
    // handle command line parameters
    let matches = parse_arguments(env::args().collect());

    // TODO: Everything below this should be analysed inside the config module

    let mut infile = if matches.free.len() == 1 {
        let path = Path::new(&matches.free[0]);
        match File::open(path) {
            Err(why) => {
                panic!("Couldn't open {}: {}",
                               path.display(), Error::description(&why))
            },
            Ok(file) => {
                info!("Opened input: {}", path.display());
                file
            }
        }
    } else {
        // TODO: check if STDOUT is a tty
        print_stderr!("Input file name missing\n");
        usage(matches.opt_present("verbose"));
        exit(1);
    };

    let mut outfile = if let Some(file) = matches.opt_str("o") {
        // try to open FILE
        let path = Path::new(&file);
        match File::create(path) {
            Err(why) => {
                panic!("Couldn't open {}: {}",
                       path.display(), Error::description(&why))
            },
            Ok(file) => {
                info!("Opened output: {}", path.display());
                file
            }
        }
    } else {
        let path = Path::new("a.z8");
        match File::create(path) {
            Err(why) => {
                panic!("Couldn't open {}: {}",
                       path.display(), Error::description(&why))
            },
            Ok(file) => {
                debug!("No output file specified, assuming default");
                info!("Opened output: {}", path.display());
                file
            }
        }
    };

    debug!("Parsed command line options");
    info!("Main started");

    // call library
    zwreec::compile(&mut infile, &mut outfile);

    info!("Main finished");
}
