# zwreec [![Build Status](https://travis-ci.org/Drakulix/zwreec.svg?branch=master)](https://travis-ci.org/Drakulix/zwreec)
*This is currently in heavy development and does not work.*

Twee to Z-Code Compiler written in the Rust programming language. This is intended to compile [interactive fiction](http://en.wikipedia.org/wiki/Interactive_fiction) in the Twee format (created by the [Twine software](http://en.wikipedia.org/wiki/Twine_(software))) to [Z-Machine](http://en.wikipedia.org/wiki/Z-machine) instructions that can be run with Z-Code interpreters like [frotz](http://frotz.sourceforge.net).

## Usage
*This uses rust nightly (currently tested with 2015-05-16). This depends on compiler plugins and can therefore not use any stable builds at the moment. Sorry for any inconvenience.*

```
Usage: target/debug/zwreec_bin [-hV] [-vq] [-l [LOGFILE]] [-o OUTPUT] INPUT

Options:
    -v --verbose        be more verbose. can be used multiple times.
    -q --quiet          be quiet
    -l --logfile [LOGFILE]
                        specify log file (default zwreec.log)
    -o FILE             name of the output file
    -h --help           display this help and exit
    -V --version        display version
```

### Build Steps
`cargo build`

Install dependencies and build the application.

`cargo test`

Run the test suite.

`cargo doc`

Create the documentation.
