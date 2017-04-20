#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate inf_4301a;

use std::str;

fuzz_target!(|data: &[u8]| {

    if let Ok(ref s) = str::from_utf8(data) {
        if let Ok(exprs) = inf_4301a::parse_expressions(&s) {
            if let Err(err) = inf_4301a::do_the_thing(exprs, &mut inf_4301a::env::Environment::new()) {
                inf_4301a::error::print_error("<fuzzer>", &s, &err);
            }
        }
    }

});
