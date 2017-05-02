//! The main file!
//!
//! Actually, this is just a wrapper / a binary that is linked to the [`compilib`] library and
//! calls the right function given certain cli arguments (see the [`main`] function)
//!
//! [`compilib`]: ../compilib/index.html
//! [`main`]: fn.main.html

extern crate compilib;

use compilib::*;

use std::env::args;

/// The main function. If an argument is provided through the command-line, the program will
/// evaluate the file. If not, it will start a nice REPL.
pub fn main() {
    let argc = args().count();
    if argc == 1 {
        repl::start();
    } else if argc == 2 {
        evaluate_file(&args().nth(1).unwrap());
    }
}
