#![feature(rustc_private)] // TODO Migrate to crates.io variant (json)
#![feature(slice_patterns)] // drool
#![feature(convert)]

extern crate hyper;
extern crate regex;
extern crate serialize;
extern crate websocket;
extern crate rustc_serialize;
extern crate ncurses;

mod authentication;
mod slack_stream;
mod display_controller;
mod user;
mod message;
mod channel;
mod current_state;
mod dispatcher;
mod view;
mod view_data;
mod input_parser;

use slack_stream::SlackStream;
use dispatcher::{Dispatcher, DispatchType};
use display_controller::DisplayController;

#[allow(dead_code)]
fn main() {
    let mut dispatcher = Dispatcher::new();
    let (token,_guard) = authentication::get_oauth_token_or_panic();


    let mut slack_stream = SlackStream::new();
    dispatcher.register_broadcaster(&mut slack_stream);

    slack_stream.establish_stream(&token);
    let initial = slack_stream.initial_state.clone().expect("Expected initial state");

    let mut display = DisplayController::new(&initial);
    dispatcher.register_subscriber(&mut display, DispatchType::RawIncomingMessage);
    dispatcher.register_subscriber(&mut display, DispatchType::ChangeCurrentChannel);
    dispatcher.register_subscriber(&mut display, DispatchType::UserInput);
    dispatcher.register_broadcaster(&mut display);

    display.start();
    dispatcher.start();

    loop {}
}
