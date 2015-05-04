extern crate zwreec;
extern crate getopts;
#[macro_use] extern crate log;
extern crate time;
extern crate term;

use std::env;
use std::fs::File;

mod utils;
use self::utils::logger;

// shorthand to display program usage
macro_rules! print_usage {
    ($prog:ident, $opts:ident) => {{
        print!("{}", $opts.usage(&format!("Usage: {} [-h] [-v] [-o OUTPUT] INPUT", $prog)));
    }}
}


fn main() {
    //early init

    //let _ = SimpleLogger::init(LogLevelFilter::Info);
    let _ = logger::CombinedLogger::init(
        vec![
            logger::FileLogger::new(logger::LogLevelFilter::Trace, File::create("zwreec.log").unwrap()),
            logger::TermLogger::new(logger::LogLevelFilter::Info),
        ]
    );

    info!("main started");

    // handling commandline parameters
    let args: Vec<String> = env::args().collect();
    let ref program = args[0];

    // define options
    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "display this help and exit");
    opts.optflag("v", "verbose", "be more verbose");
    opts.optopt("o", "", "name of the output file", "FILE");

    let parsed_opts = match opts.parse(&args[1..]) {
        Ok(m)  => { m }
        Err(f) => {
            // parsing error
            // display usage and return
            println!("{}", f.to_string());
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

    if parsed_opts.opt_present("v") {
        // parsed "-v|--verbose"
        // set loglevel to verbose
        // utils::log::LOG_LEVEL = utils::log::LogLevel::VERBOSE;
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
        println!("Input file name missing");
        print_usage!(program, opts);
        // TODO: figure out a way to set exit code
        return;
    };


    // call library
    zwreec::compile(&infile, &outfile);

    // only for testing
    debug!("(1) {}", zwreec::frontend::temp_hello());
    debug!("(2) {}", zwreec::backend::temp_hello());
    debug!("(3) {}", zwreec::file::temp_hello());

    info!("main finished");
}
