#![feature(rustc_private)] // TODO Migrate to crates.io variant (json)
#![feature(core)]
#![feature(convert)] // & instead of as_ref/as_slice

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

    let slack_stream = messages_stream::establish_stream(&token);
    let current_state = new_current_state_from_json(&slack_stream.initial_state);
    for message in slack_stream.iter() {
        match new_message_from_str(&message) {
            Ok(message) => {
                match message.event_type.unwrap_or("".to_string()).as_ref() {
                    "message" => {
                        match current_state.user_id_to_user(&message.user.unwrap_or("".to_string())) {
                            Some(user) => {
                                println!("{} - {}: {}", message.channel.unwrap(), user.name, message.text.unwrap())
                            },
                            None => {
                                println!("{} - {}: {}", message.channel.unwrap(), "UnknownUser", message.text.unwrap())
                            }

                        }
                    },
                    _ => println!("{}", message.text.unwrap_or("".to_string()))
                }
            },
            Err(e) => println!("{}", e)
        }
    } 
}
