use std::env;
use std::fs::File;

mod utils;
use self::utils::logger;

extern crate zwreec;
#[macro_use] extern crate log;
extern crate time;
extern crate term;

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

    let mut input_file_name: &str;
    let mut output_file_name: &str;

    match args.len() {
        1 => {
            input_file_name = "a.in";
            output_file_name = "a.out";
        },
        3 => {
            input_file_name = &args[1];
            output_file_name = &args[2];
        },
        _ => {
            help();
            return;
        }
    };

    // call library
    zwreec::compile(input_file_name, output_file_name);

    // only for testing
    debug!("(1) {}", zwreec::frontend::temp_hello());
    debug!("(2) {}", zwreec::backend::temp_hello());
    debug!("(3) {}", zwreec::file::temp_hello());

    info!("main finished");
}

fn help() {
    error!("invalid arguments");
    info!("usage:\n    zwreec <input_file> <output_file>");
}
