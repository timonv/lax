#![feature(rustc_private)] // TODO Migrate to crates.io variant (json)
#![feature(core)]
#![feature(convert)]

use std::thread;

extern crate hyper;
extern crate regex;
extern crate serialize;
extern crate websocket;
extern crate rustc_serialize;

mod authentication;
mod messages_stream;
mod user_view;
mod user;
mod message;
mod channel;
mod current_state;

/*
 *  messages_stream <--> main <---> current_state --> user_view
 *                        ^                               |
 *                        \-------------------------------/
 */
#[allow(dead_code)]
fn main() {
    let (token,_guard) = authentication::get_oauth_token_or_panic();

    let slack_stream = messages_stream::establish_stream(&token);
    let current_state = current_state::new_from_str(&slack_stream.initial_state);
    let (user_view, input_endpoint) = user_view::start();

    thread::spawn(move || {
        for input in input_endpoint.iter() {
            println!("ECHO: {}", input.trim())
        }
    });

    for raw_message in slack_stream.iter() {
        match current_state.parse_incoming_message(&raw_message) {
            Ok(message) => user_view.incoming_message(message).ok().expect("Could not send message to view"),
            Err(e) => println!("ERROR PARSING: {}\n{}", e, raw_message)
        }
    }
}
