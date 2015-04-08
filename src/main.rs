#![feature(rustc_private)] // TODO Migrate to crates.io variant (json)
#![feature(core)]
#![feature(convert)]

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

#[allow(dead_code)]
fn main() {
    let (token,_guard) = authentication::get_oauth_token_or_panic();

    let slack_stream = messages_stream::establish_stream(&token);
    let current_state = current_state::new_from_str(&slack_stream.initial_state);
    for raw_message in slack_stream.iter() {
        match current_state.parse_incoming_message(&raw_message) {
            Ok(message) => println!("{}", message),
            Err(e) => println!("ERROR PARSING: {}\n{}", e, raw_message)
        }
    } 
}
