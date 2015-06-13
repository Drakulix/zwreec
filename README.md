# zwreec [![Build Status](https://travis-ci.org/Drakulix/zwreec.svg?branch=master)](https://travis-ci.org/Drakulix/zwreec)

<img width=144px src="https://dl.dropboxusercontent.com/u/70410095/zwreec/logo.png">
*Logo by [@madmalik](https://github.com/madmalik)*

Twee to Z-Code Compiler written in the Rust programming language. This is intended to compile [interactive fiction](http://en.wikipedia.org/wiki/Interactive_fiction) in the Twee format (created by the [Twine software](http://en.wikipedia.org/wiki/Twine_(software))) to [Z-Machine](http://en.wikipedia.org/wiki/Z-machine) instructions that can be run with Z-Code interpreters like [frotz](http://frotz.sourceforge.net).

## Installation
*This uses rust nightly (currently tested with 2015-06-02). This depends on compiler plugins and can therefore not use any stable builds at the moment. Sorry for any inconvenience.*

To install the current rust version run:

```curl -s https://static.rust-lang.org/rustup.sh | sh -s -- --channel=nightly --date=2015-06-02```

## Quick Start
Install rust, clone the repository, then run:
```
cargo build
```

Compile the ASCII sample file:
```
./target/debug/zwreec ./tests/integration/sample/ASCII.twee -o ASCII.z8
```

_Edit the above line to compile different twee adventures._

Then you can run `./ASCII.z8` with your favorite Z-Code interpreter, like [frotz](http://frotz.sourceforge.net).

## Features
Only rudimentary Twee features are supported right now. This is about to change in the upcoming weeks. Check the github issues for more information on the currently supported features.

## Usage
```
Usage: target/debug/zwreec [-hV] [-vq] [-l [LOGFILE]] [-o OUTPUT] INPUT

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

_For a release build:_
`cargo build --release`

`cargo test`

Run the test suite.

`cargo doc`

Create the documentation.
