use getopts;

use std::vec::Vec;

pub struct Config {
    pub req_input: bool,
    pub testmode: bool,
    pub test_cases: Vec<TestCase>,
}

/// Returns a `Config` struct with preset defaults
pub fn default_config() -> Config {
    Config{
        req_input: true,
        testmode: false,
        test_cases: Vec::new(),
    }
}

pub enum TestCase {
    ZcodeBackend,
}


/// Appends a `getopts::Options` with zwreeclib specific flags
pub fn zwreec_options(mut opts: getopts::Options) -> getopts::Options {
    opts.optflag("", "test-zcode", "tests the zcode backend by running all implemented opcodes once.");

    opts
}


pub fn parse_matches(matches: &getopts::Matches) -> Config {
    // load defaults
    let mut cfg = default_config();

    // parse options that skip the normal pipeline first, might speed things 
    // up a bit.
    if matches.opt_present("test-zcode") {
        cfg.req_input = false;
        cfg.testmode = true;
        cfg.test_cases.push(TestCase::ZcodeBackend);
    }

    cfg
}
