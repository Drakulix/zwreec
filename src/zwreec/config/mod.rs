use getopts;

/// Returns the primary options for zwreec. It is e.g. used to generate the 
/// usage information.
pub fn zwreec_short_options() -> getopts::Options {
    let mut opts = getopts::Options::new();
    opts.optflagmulti("v", "verbose", "be more verbose. Can be used multiple times.");
    opts.optflag("q", "quiet", "be quiet");
    opts.optflagopt("l", "logfile", "specify log file (default zwreec.log)", "LOGFILE");
    opts.optopt("o", "", "name of the output file", "FILE");
    opts.optflag("h", "help", "display this help and exit");
    opts.optflag("V", "version", "display version");

    opts
}

/// Returns all options for zwreec.
pub fn zwreec_options() -> getopts::Options {
    let mut opts = zwreec_short_options();

    opts
}
