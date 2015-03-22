#![feature(io)]
#![feature(old_io)]
#![feature(fs)]
#![feature(rustc_private)] // TODO Migrate to crates.io variant (json)
#![feature(net)]
#![feature(core)]

extern crate hyper;
extern crate regex;
extern crate serialize;
extern crate websocket;

mod authentication;
mod messages_stream;
mod user_view;

// TODO Switch to new (easy)
use std::old_io::stdin;

#[allow(dead_code)]
fn main() {
    let (token,_guard) = authentication::get_oauth_token_or_panic();

    let slack_stream = messages_stream::establish_stream(&token);
    let mut view = user_view::new();
    // interface.update_state(slack_stream.initial_state());
    // TODO Clear screen here
    
    println!("Connection established!");

    for message in slack_stream.into_iter() {
        view.print_message(message);
    }

    println!("Server closed!")
}
