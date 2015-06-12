use getopts;

use std::vec::Vec;

pub struct Config {
    pub easter_egg: bool,
    pub test_cases: Vec<TestCase>,
}

/// Returns a `Config` struct with preset defaults
pub fn default_config() -> Config {
    Config{
        easter_egg: true,
        test_cases: Vec::new(),
    }
}

// TODO: If this stays only one Test Case, enum should be removed
pub enum TestCase {
    ZcodeBackend,
}


/// Appends a `getopts::Options` with zwreeclib specific flags
pub fn zwreec_options(mut opts: getopts::Options) -> getopts::Options {
    opts.optmulti("F", "feature", "", "FEAT");
    opts.optmulti("N", "no-feature", "enable or disable a feature (can occur multiple times).
                        List of supported features (default):
                            easter-egg (enabled)", "FEAT");
    opts.optflag("e", "generate-sample-zcode", "writes out a sample zcode file, input file is not used and can be omitted");

    opts
}


pub fn parse_matches(matches: &getopts::Matches) -> Config {
    // load defaults
    let mut cfg = default_config();

    if matches.opt_present("generate-sample-zcode") {
        cfg.test_cases.push(TestCase::ZcodeBackend);
    }

    // TODO: Find a way to make these two loops somewhat less.. repetitive
    for s in matches.opt_strs("F") {
        match s.as_ref() {
            "easter-egg" => {
                 cfg.easter_egg = true;
                 debug!("enabled easter-egg");
            },
            _ => {
                error!("Cannot enable feature {} - feature not known.", s);
            }
        }
    }

    for s in matches.opt_strs("N") {
        match s.as_ref() {
            "easter-egg" => {
                cfg.easter_egg = false;
                debug!("disabled easter-egg");
            },
            _ => {
                error!("Cannot disable feature {} - feature not known.", s);
            }
        }
    }

    cfg
}
