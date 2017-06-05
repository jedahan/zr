#![cfg_attr(feature="clippy", feature(plugin))]

#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate libc;
extern crate libzr;

use std::io::Write;

fn main() {
    if let Err(err) = libzr::run() {
        writeln!(&mut std::io::stderr(), "{}", err)
            .expect("error writing to stderr");
        std::process::exit(libc::EXIT_FAILURE);
    }
}
