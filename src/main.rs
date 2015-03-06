#![feature(io)]
#![feature(old_io)]
#![feature(fs)]
#![feature(rustc_private)] // TODO Migrate to crates.io variant (json)
#![feature(net)]
#![feature(core)]

extern crate hyper;
extern crate regex;
extern crate serialize;

mod authentication;

#[allow(dead_code)]
fn main() {
    let (token,_guard) = authentication::get_oauth_token_or_panic();
    println!("{}", token);
}
