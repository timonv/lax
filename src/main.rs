#![feature(rustc_private)] // TODO Migrate to crates.io variant (json)
#![feature(core)]

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

use current_state::new_current_state_from_json;
use message::new_message_from_str;

#[allow(dead_code)]
fn main() {
    let (token,_guard) = authentication::get_oauth_token_or_panic();

    // let slack_stream = messages_stream::establish_stream(&token);
    // let current_state = new_current_state_from_json(&slack_stream.initial_state);
    // for message in slack_stream.iter() {
    //     match new_message_from_str(message.as_slice()) {
    //         Ok(message) => {
    //             match current_state.user_id_to_user(message.user.as_slice()) {
    //                 Some(user) => {
    //                     println!("{} - {}: {}", message.channel, user.name, message.text)
    //                 },
    //                 None => {
    //                     println!("{} - {}: {}", message.channel, "UnknownUser", message.text)
    //                 }

    //             }
    //         },
    //         Err(e) => println!("{}", e)
    //     }
    // } 
}
