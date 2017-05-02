#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate compilib;

use std::str;

fuzz_target!(|data: &[u8]| {

    if let Ok(ref s) = str::from_utf8(data) {
        if let Ok(exprs) = compilib::parse_expressions(&s) {
            if let Err(err) = compilib::do_the_thing(exprs, &mut compilib::env::Environment::new()) {
                compilib::error::print_error("<fuzzer>", &s, &err);
            }
        }
    }

});
