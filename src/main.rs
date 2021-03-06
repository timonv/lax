#![feature(slice_patterns)] // drool
#![feature(plugin)]
#![feature(phase)]
#![plugin(json_macros)]
#![feature(convert)]
#![feature(scoped)] // debugging > safety
#![feature(rustc_private)]

extern crate hyper;
extern crate regex;
extern crate websocket;
extern crate rustc_serialize;
extern crate ncurses;
extern crate rdispatcher;
#[macro_use]
extern crate log;
extern crate env_logger;

mod authentication;
mod channel;
mod current_state;
mod dispatch_type;
mod display_controller;
mod input_parser;
mod message;
mod slack_stream;
mod user;
mod view;
mod view_data;
#[cfg(test)] mod test_helpers;

use slack_stream::SlackStream;
use display_controller::DisplayController;
use dispatch_type::DispatchType;
use rdispatcher::Dispatcher;
use std::thread;

#[allow(dead_code)]
fn main() {
    env_logger::init().ok().expect("[main] Could not start logger");
    
    info!("Started lax");

    let mut dispatcher = Dispatcher::new();
    let (token,_guard) = authentication::get_oauth_token_or_panic();


    let mut slack_stream = SlackStream::new();

    dispatcher.register_broadcaster(&mut slack_stream);
    slack_stream.establish_stream(&token);
    dispatcher.register_subscriber(&mut slack_stream, DispatchType::UserInput); // wtf

    let initial = slack_stream.initial_state.clone().expect("Expected initial state");
    let mut display = DisplayController::new(&initial);
    dispatcher.register_subscriber(&mut display, DispatchType::RawIncomingMessage);
    dispatcher.register_subscriber(&mut display, DispatchType::ChangeCurrentChannel);
    dispatcher.register_subscriber(&mut display, DispatchType::ListChannels);
    dispatcher.register_subscriber(&mut display, DispatchType::UserInput); // wtf
    dispatcher.register_broadcaster(&mut display);

    display.start();
    dispatcher.start();

    thread::park();
}
