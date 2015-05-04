extern crate zwreec;
extern crate getopts;
#[macro_use] extern crate log;
extern crate time;
extern crate term;

use std::env;
use std::vec::Vec;
use std::fs::File;
use std::io::Write;

mod utils;
use self::utils::logger;

// shorthand to display program usage
macro_rules! print_usage {
    ($prog:ident, $opts:ident) => {{
        print_stderr!("{}", $opts.usage(&format!("Usage: {} [-hV] [-vq] [-l [LOGFILE]] [-o OUTPUT] INPUT", $prog)));
    }}
}

// Found in:
// http://stackoverflow.com/a/27590832
macro_rules! print_stderr(
    ($($arg:tt)*) => (
        match write!(&mut ::std::io::stderr(), $($arg)* ) {
            Ok(_) => {},
            Err(x) => panic!("Unable to write to stderr: {}", x),
        }
    )
);

fn main() {
    //early init

    // handling commandline parameters
    let args: Vec<String> = env::args().collect();
    let ref program = args[0];
    let mut loggers: Vec<Box<logger::SharedLogger>> = vec![];

    // define options
    let mut opts = getopts::Options::new();
    opts.optflagmulti("v", "verbose", "be more verbose. can be used multiple times.");
    opts.optflag("q", "quiet", "be quiet");
    opts.optflagopt("l", "logfile", "specify log file (default zwreec.log)", "LOGFILE");
    opts.optopt("o", "", "name of the output file", "FILE");
    opts.optflag("h", "help", "display this help and exit");
    opts.optflag("V", "version", "display version");

    let parsed_opts = match opts.parse(&args[1..]) {
        Ok(m)  => { m }
        Err(f) => {
            // parsing error
            // display usage and return
            print_stderr!("{}\n", f.to_string());
            print_usage!(program, opts);
            // TODO: figure out a way to set exit code
            return;
        }
    };

    // examinate options
    if parsed_opts.opt_present("h") {
        // parsed "-h|--help"
        // display usage and return
        print_usage!(program, opts);
        return;
    }

    if parsed_opts.opt_present("V") {
        // parsed "-V|--version"
        // display current version
        println!("{} {}", program, match option_env!("CFG_VERSION") {
            Some(s) => s.to_string(),
            None => format!("{}.{}.{}{}",
                            env!("CARGO_PKG_VERSION_MAJOR"),
                            env!("CARGO_PKG_VERSION_MINOR"),
                            env!("CARGO_PKG_VERSION_PATCH"),
                            option_env!("CARGO_PKG_VERSION_PRE").unwrap_or(""))
        });
        return;
    }

    if parsed_opts.opt_present("v") {
        // parsed "-v|--verbose"
        // set loglevel to verbose
        loggers.push(logger::TermLogger::new(
                match parsed_opts.opt_count("v") {
                    1 => logger::LogLevelFilter::Info, 
                    2 => logger::LogLevelFilter::Debug, 
                    _ => logger::LogLevelFilter::Trace,
                }));
    } else if parsed_opts.opt_present("q") {
        // parsed "-q|--quiet"
        // set loglevel to error
        loggers.push(logger::TermLogger::new(logger::LogLevelFilter::Error));
    } else {
        // default
        // set loglevel to warn
        loggers.push(logger::TermLogger::new(logger::LogLevelFilter::Warn));
    }

    if parsed_opts.opt_present("l") {
        // parsed "-l|--logfile"
        // sets a logger to output to logfile
        let name = if let Some(n) = parsed_opts.opt_str("l") {
            n
        } else {
            "zwreec.log".to_string()
        };
        loggers.push(logger::FileLogger::new(
                        logger::LogLevelFilter::Trace, 
                        File::create(name).unwrap())
            );
    }

    let outfile = if let Some(file) = parsed_opts.opt_str("o") { 
        // parsed "-o FILE"
        // set of to filename
        file
    } else {
        // NOTE: string manipulation in rust is stil weird.
        let mut s = String::new();
        s.push_str("a.out");
        s
    };

    let infile = if parsed_opts.free.len() == 1 {
         // check number of 'free' parameter
         // one free parameter is the input file name
         parsed_opts.free[0].clone()
    } else {
        print_stderr!("Input file name missing\n");
        print_usage!(program, opts);
        // TODO: figure out a way to set exit code
        return;
    };

    // activate logger
    // NOTE: I am not sure, if there is a *good* way to do this earlier

    //let _ = SimpleLogger::init(LogLevelFilter::Info);
    let _ = logger::CombinedLogger::init(loggers);

    debug!("parsed command line options");
    info!("main started");
    warn!("WARN");

    // call library
    zwreec::compile(&infile, &outfile);

    // only for testing
    debug!("(1) {}", zwreec::frontend::temp_hello());
    debug!("(2) {}", zwreec::backend::temp_hello());
    debug!("(3) {}", zwreec::file::temp_hello());

    info!("main finished");
}
