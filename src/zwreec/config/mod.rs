//! Configuration for the zwreec compiler.
//!
//! Uses the `getopts` library to append a `getopts::Options` with compiler 
//! specific flags and then creates a `Config` struct from `getopts::Matches`.
//!
//! # Usage
//!
//! This module depends on the [getopts](http://doc.rust-lang.org/getopts/getopts/index.html)
//! crate. To use it, you need to add `getopts` to your project. Read getopts'
//! [Usage](http://doc.rust-lang.org/getopts/getopts/index.html#usage) for more information.
//!
//! # Example
//!
//! The following example shows how to generate a `Config` struct from an empty
//! `getopts::Options`.
//!
//! ```
//! extern crate zwreec;
//! extern crate getopts;
//!
//! use zwreec::config;
//! use zwreec::config::Config;
//! 
//!
//! fn main() {
//!     let args: Vec<String> = std::env::args().collect();
//!     let opts = config::zwreec_options(getopts::Options::new());
//!
//!     let matches = match opts.parse(&args[1..]) {
//!         Ok(m) => m,
//!         Err(f) => { panic!(f.to_string()) }
//!     };
//!
//!     let cfg = Config::from_matches(&matches);
//! }
//! ```
//!
//! # Appending this module (Development)
//!
//! To add new compiler flags for the compiler, you need to change three parts 
//! in this module.
//!
//! 1. Appending the Struct
//!
//!     To store your new setting, you need to add it as a field to the `Config` struct. Here we
//!     add a new boolean `pub italics: bool` to control *italics*. Always add a field-comment.
//!
//!     ```ignore
//!     pub struct Config {
//!         /// Add easter egg to compiler
//!         pub easter_egg: bool,
//!         /// Instruct compiler to run these test-cases
//!         pub test_cases: Vec<TestCase>,
//!         /// Enables or disalbes Italics
//!         pub italics: bool,
//!     }
//!     ```
//!
//!     Now you also need to change `Config::default_config()` to provide the
//!     default value for your new field:
//!
//!     ```ignore
//!     pub fn default_config() -> Config {
//!         Config{
//!             easter_egg: true,
//!             test_cases: Vec::new(),
//!             italics: true,
//!         }
//!     }
//!     ```
//!
//! 2. Parse your flag in `from_matches()`
//!
//!     `from_matches` uses the Options `-F` and `-N` to enable or disable boolean flags.
//!     Therefore, if you added a new boolean flag, you can now simply add it to the match
//!     statement.
//!
//!     ```ignore
//!     for s in matches.opt_strs("F") {
//!         match s.as_ref() {
//!             "easter-egg" => {
//!                  cfg.easter_egg = true;
//!                  debug!("enabled easter-egg");
//!             },
//!             "italics" => {
//!                 cfg.italics = true;
//!                 debug!("enabbled italics");
//!             },
//!             _ => {
//!                 error!("Cannot enable feature {} - feature not known.", s);
//!             }
//!         }
//!     }
//!
//!     for s in matches.opt_strs("N") {
//!         match s.as_ref() {
//!             "easter-egg" => {
//!                 cfg.easter_egg = false;
//!                 debug!("disabled easter-egg");
//!             },
//!             "italics" => {
//!                 cfg.italics = false;
//!                 debug:("disalbed italics");
//!             }
//!             _ => {
//!                 error!("Cannot disable feature {} - feature not known.", s);
//!             }
//!         }
//!     }
//!     ```
//!
//! 3. Adding something that is not a boolean
//!
//!     If you need to add an option that is *not* a boolean, you will need to append the struct
//!     and `default_config()` accordingly (e.g. `pub notaflag: String,`). Then you need to add a new
//!     `getopts::Option` to `zwreec_options()`:
//!
//!     ```ignore
//! pub fn zwreec_options(mut opts: getopts::Options) -> getopts::Options {
//!     opts.optmulti("F", "feature", "", "FEAT");
//!     opts.optmulti("N", "no-feature", "enable or disable a feature (can occur multiple times).
//!                         List of supported features (default):
//!                             easter-egg (enabled)", "FEAT");
//!     opts.optflag("e", "generate-sample-zcode", "writes out a sample zcode file, input file is not used and can be omitted");
//!
//!     opts.optopt("n", "notaflag", "notaflag", "SOMETHING");
//! 
//!     opts
//! }
//!     ```
//!
//!     Now you can append `Config::from_matches()` to analyse the provided matches for your new
//!     option and set the `Config` accordingly:
//!
//!     ```ignore
//!     if let s = matches.opt_str("n") {
//!         cfg.notaflag = s;
//!     }
//!     ```
//!
//! 4. Now you can use the new flag inside the compiler.
use getopts;

use std::vec::Vec;


/// Represents the configuration for the compiler.
///
/// This struct is created using either `config::default_config()` or 
/// `config::parse_matches(matches: &getopts::Matches)`. It contains a series
/// of fields that can be used to control the behaviour of the compiler.
///
/// # Example Usage inside the compiler
///
/// ```
/// # use zwreec::config;
/// let cfg = config::Config::default_config();
///
/// if cfg.easter_egg {
///     println!("Egg!");
/// }
/// ```
#[derive(Clone)]
pub struct Config {
    /// force a bright background and dark text
    pub bright_mode: bool,
    /// Add easter egg to compiler
    pub easter_egg: bool,
    /// Force compilation despite errors
    pub force: bool,
    pub force_unicode: bool,
    pub half_memory: bool,
    pub no_colours: bool,
    pub no_unicode: bool,

    /// Instruct compiler to run these test-cases
    pub test_cases: Vec<TestCase>,
}

impl Config {
    /// Returns a `Config` struct with preset defaults
    ///
    /// # Example
    ///
    /// ```
    /// use zwreec::config::Config;
    /// let cfg = Config::default_config();
    /// ```
    pub fn default_config() -> Config {
        Config{
            bright_mode: false,
            easter_egg: true,
            force: false,
            force_unicode: false,
            half_memory: false,
            no_colours: false,
            no_unicode: false,
            test_cases: Vec::new(),
        }
    }

    /// Returns a `Config` struct by using `getopts::Matches` to set the fields.
    ///
    /// This method analyses a `getopts::Matches` for fields provided by 
    /// `zwreec_options()`. 
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate getopts;
    /// # extern crate zwreec;
    /// let args: Vec<String> = std::env::args().collect();
    /// let opts = zwreec::config::zwreec_options(getopts::Options::new());
    ///
    /// let matches = match opts.parse(&args[1..]) {
    ///     Ok(m) => m,
    ///     Err(f) => { panic!(f.to_string()) }
    /// };
    ///
    /// let cfg = zwreec::config::Config::from_matches(&matches);
    /// ```
    pub fn from_matches(matches: &getopts::Matches) -> Config {
        // load defaults
        let mut cfg = Config::default_config();

        if matches.opt_present("generate-sample-zcode") {
            cfg.test_cases.push(TestCase::ZcodeBackend);
        }

        if matches.opt_present("f") {
            cfg.force = true;
        }

        // TODO: Find a way to make these two loops somewhat less.. repetitive
        for s in matches.opt_strs("F") {
            match s.as_ref() {
                "bright-mode" => {
                     cfg.bright_mode = true;
                     debug!("enabled bright-mode");
                },
                "easter-egg" => {
                     cfg.easter_egg = true;
                     debug!("enabled easter-egg");
                },
                "force-unicode" => {
                     cfg.force_unicode = true;
                     debug!("enabled force-unicode");
                },
                "no-colours" => {
                     cfg.no_colours = true;
                     debug!("enabled no-colours");
                },
                "half-memory" => {
                     cfg.half_memory = true;
                     debug!("enabled half-memory");
                },
                "no-unicode" => {
                     cfg.no_unicode = true;
                     debug!("enabled no-unicode");
                },
                _ => {
                    error!("Cannot enable feature {} - feature not known.", s);
                }
            }
        }

        for s in matches.opt_strs("N") {
            match s.as_ref() {
                "bright-mode" => {
                     cfg.bright_mode = false;
                     debug!("disabled bright-mode");
                },
                "easter-egg" => {
                    cfg.easter_egg = false;
                    debug!("disabled easter-egg");
                },
                "force-unicode" => {
                     cfg.force_unicode = false;
                     debug!("disabled force-unicode");
                },
                "no-colours" => {
                     cfg.no_colours = false;
                     debug!("disabled no-colours");
                },
                "half-memory" => {
                     cfg.half_memory = false;
                     debug!("disabled half-memory");
                },
                "no-unicode" => {
                     cfg.no_unicode = false;
                     debug!("disabled no-unicode");
                },
                _ => {
                    error!("Cannot disable feature {} - feature not known.", s);
                }
            }
        }

        cfg
    }
}

// TODO: If this stays only one Test Case, enum should be removed
/// The Type used to define backend tests for the compiler
#[derive(PartialEq,Clone)]
pub enum TestCase {
    /// Skips the normal compiler chain and builds an example zcode file by 
    /// using every opcode.
    ZcodeBackend,
}


/// Appends a `getopts::Options` with compiler specific flags
///
/// The method `Config::from_matches()` looks for very specific `getopts::Matches`.
/// This function takes an `getopts::Options` to append it with Options required 
/// by `from_matches`. It currently adds three fields:
///
/// ```ignore
/// opts.optmulti("F", "feature", "", "FEAT"); 
/// opts.optmulti("N", "no-feature", "enable or disable a feature (can occur multiple times).
///                     List of supported features (default):
///                         easter-egg (enabled)", "FEAT");
/// opts.optflag("e", "generate-sample-zcode", "writes out a sample zcode file, input file is not used and can be omitted");
/// ```
///
/// # Example
///
/// You can use this function to append your `getopts::Options`.
///
/// ```
/// # extern crate getopts;
/// # extern crate zwreec;
///
/// let mut opts = getopts::Options::new();
/// opts.optflag("h", "help", "print this message");
/// 
/// let opts = zwreec::config::zwreec_options(opts);
/// ```
///
/// Another useful example is to use it to gernerate a more compact usage by
/// having a function that only returns your options.
///
/// ```
/// # extern crate getopts;
/// # extern crate zwreec;
///
/// fn options() -> getopts::Options {
///     let mut opts = getopts::Options::new();
///     opts.optflag("h", "help", "display this help and exit");
///     opts.optflag("V", "version", "display version");
/// 
///     opts
/// }
///
/// fn print_usage(program: &str, verbose: bool) {
///     let brief = format!("Usage: {} [options]", program);
///
///     let opts = if verbose {
///         zwreec::config::zwreec_options(options())
///     } else {
///         options()
///     };
///
///     print!("{}", opts.usage(&brief));
/// }
/// ```
/// As you can see, `options()` returns your own command line options, which are then conditionally
/// expanded by using `zwreec_options()`.
pub fn zwreec_options(mut opts: getopts::Options) -> getopts::Options {
    opts.optflag("f", "force", "Try ignoring any errors that may occur and generate Z-Code anyways.
        This feature is highly unstable and may lead to corrupt output files.");
    opts.optmulti("F", "feature", "", "FEAT");
    opts.optmulti("N", "no-feature", "Enable or disable a feature (can occur multiple times).
        For more information about the supported features run --help with -v and see the feature
        list at the end of the output", "FEAT");
    opts.optflag("e", "generate-sample-zcode", "Write out a sample zcode file, input file is not used and can be omitted");

    opts
}

/// Prints a usage
///
/// This takes your options and prints a usage for those options.
/// It also includes zwreec_options and a feature list if a verbose usage was requested.
pub fn zwreec_usage(verbose: bool, mut opts: getopts::Options, brief: &str) -> String {
    use std::fmt::format;

    if verbose {
        opts = zwreec_options(opts);
    }

    let options_usage = opts.usage(brief);

    let features_usage = if verbose {
        "List of supported features (default value in parenthesis)
    bright-mode (disabled)
        Enables a bright background and a dark text color
    easter-egg (enabled) 
        Enables the generation of easter egg code. Enter the secret combination
        in your Z-Machine interpreter to activate the easter egg. This requires
        some extra space - disable this if your output file is getting too large
    force-unicode (disabled)
        Force the generation of print_unicode opcodes every time a unicode
        character is encountered. This disables the generation of the unicode
        translation table
    half-memory (disabled)
        Cut down space for static variable strings and heap in order to have
        binaries probably smaller than 64kB as only DZIP32.exe on DOS can handle
        larger files, but DZIP.exe has a limit on 64kB. If your file is still
        large, consider disabling the easter-egg flag
    no-colours (disabled)
        Suppress generation of set_colour and set_text_style opcodes and disable
        the colour bit in the second byte of the header - this is required for
        some old interpreters like for DZIP on DOS/Atari
    no-unicode (disabled)
        Replaces opcode print_unicode with print_char to let it run on
        interpreters without unicode support like JZIP"
    } else {
        "Additional help:
    --help -v           Print the full set of options zwreec accepts"
    };

    format(format_args!("{}\n{}\n", options_usage, features_usage.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use getopts;

    fn config_from_args(args: Vec<String>) -> Config {
        let opts = zwreec_options(getopts::Options::new());

        match opts.parse(&args) {
            Ok(m) => Config::from_matches(&m),
            Err(f) => { panic!(f.to_string()) }
        }
    }

    #[test]
    fn test_feature_easter_egg_true() {
        let cfg = config_from_args(vec!["-F".to_string(), "easter-egg".to_string()]);

        assert_eq!(cfg.easter_egg, true);
    }

    #[test]
    fn test_feature_easter_egg_false() {
        let cfg = config_from_args(vec!["-N".to_string(), "easter-egg".to_string()]);

        assert_eq!(cfg.easter_egg, false);
    }

    #[test]
    fn test_feature_easter_egg_both() {
        let cfg = config_from_args(vec![
                                   "-N".to_string(),
                                   "easter-egg".to_string(),
                                   "-F".to_string(),
                                   "easter-egg".to_string()]);

        assert_eq!(cfg.easter_egg, false);
    }

    #[test]
    fn test_feature_bright_mode_true() {
        let cfg = config_from_args(vec!["-F".to_string(), "bright-mode".to_string()]);

        assert_eq!(cfg.bright_mode, true);
    }

    #[test]
    fn test_feature_bright_mode_false() {
        let cfg = config_from_args(vec!["-N".to_string(), "bright-mode".to_string()]);

        assert_eq!(cfg.bright_mode, false);
    }

    #[test]
    fn test_generate_sample_zcode() {
        let cfg = config_from_args(vec!["-e".to_string()]);

        assert_eq!(cfg.test_cases.is_empty(), false);

        let mut contains = false;
        for tc in cfg.test_cases {
            if tc == TestCase::ZcodeBackend {
                contains = true;
            }
        }

        assert!(contains);
    }
}
