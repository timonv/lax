// Sets up a connections to the websocket stream
// I guess the only way to properly write unit tests for this
// is to make the websockets api generic... bad idea, not
// with unstable 3rd parties.

// std
use std::io::prelude::*;
use serialize::json;
use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::{Arc, RwLock};

// extern
use hyper::client::Client as HttpClient;
use websocket::{Message, Client};
use websocket::client::sender::Sender as WssSender;
use websocket::Sender as WssSenderTrait;
use websocket::client::receiver::Receiver as WssReceiver;
use websocket::Receiver as WssReceiverTrait;
// use websocket::Sender as WssSender;
// use websocket::Receiver as WssReceiver;
use websocket::dataframe::DataFrame;
use websocket::client::request::Url;
use websocket::stream::WebSocketStream;

// TODO Use this
// const SLACK_RTM_START: &'static str = "https://slack.com/api/rtm.start?token={}";

// ... not really a stream
// More like a guard with extras
pub struct SlackStream<'a> {
    // TODO Figure out why I need to pass the return type
    // of the thread here as well. fml
    _receiver_guard: thread::JoinGuard<'a, ()>,
    _sender_guard: thread::JoinGuard<'a, ()>,
    _message_sender: Sender<Message>,
}

pub fn establish_stream(authtoken: &str) -> SlackStream  {
    // TODO Place all my pretty data in a struct
    let json = request_realtime_messaging(authtoken);

    // As per example
    let wss_url = Url::parse(json.find("url").unwrap().as_string().unwrap()).unwrap();
    let request = Client::connect(wss_url).unwrap();
    let response = request.send().unwrap(); // send request and retrieve response
    response.validate().unwrap(); // validate the response (check wss frames, idk?)

    let (mut sender, mut receiver) = response.begin().split();
    let (tx, rx) = channel::<Message>();

    let send_guard = spawn_send_loop(sender, rx);
    let receiver_guard = spawn_receive_loop(receiver, tx.clone());

    SlackStream {
        _receiver_guard: receiver_guard,
        _sender_guard: send_guard,
        _message_sender: tx,
    }
}

fn request_realtime_messaging(authtoken: &str) -> json::Json {
    let mut client = HttpClient::new();
    let mut res = client.get(format!("https://slack.com/api/rtm.start?token={}", authtoken).as_slice()).send().unwrap();
    let mut body = String::new();
    res.read_to_string(&mut body);

    json::from_str(&body).unwrap()
}

fn spawn_send_loop<'a>(mut wss_sender: WssSender<WebSocketStream>, message_receiver: Receiver<Message>) -> thread::JoinGuard<'a, ()> {
    let send_guard = thread::scoped(move || {
        loop {
            let message = match message_receiver.recv() {
                Ok(m) => m,
                Err(e) => {
                    println!("Send Loop: {:?}", e);
                    return;
                }
            };

            match message {
                Message::Close(_) => {
                    let _ = wss_sender.send_message(message);
                    return;
                }
                _ => (),
            }

            match wss_sender.send_message(message) {
                Ok(()) => (),
                Err(e) => {
                    println!("Send Loop: {:?}", e);
                    let _ = wss_sender.send_message(Message::Close(None));
                    return;
                }
            }

        }
    });

    send_guard
}

fn spawn_receive_loop<'a>(mut wss_receiver: WssReceiver<WebSocketStream>, message_sender: Sender<Message>) -> thread::JoinGuard<'a, ()> {
    // Double thread to keep receiver alive
    // Previously crashed because ssl
    thread::scoped(move || {
        let arw_wss = Arc::new(RwLock::new(wss_receiver));
        loop {
            let arw_wss = arw_wss.clone();
            let message_sender_for_thread = message_sender.clone();
            let guard = thread::spawn(move || {
                let mut wss_receiver = match arw_wss.write() {
                    Ok(v) => v,
                    Err(e) => {
                        println!("Received error message! {:?}", e);
                        return;
                    }
                };
                for message in wss_receiver.incoming_messages::<Message>() {
                    let message = match message {
                        Ok(m) => m,
                        Err(e) => {
                            println!("Received error message: {:?}", e);
                            break; // Ignore, next iter
                        }
                    };

                    match message {
                        Message::Close(_) => {
                            let _ = match message_sender_for_thread.send(Message::Close(None)) {
                                Ok(()) => {
                                    println!("Received close message and server closed!");
                                    return; // Closing thread
                                },
                                Err(e) => {
                                    println!("Receive failed close message: {:?}", e);
                                    return; // Closing thread
                                }
                            };
                        },
                        // Ping keeps the connection alive
                        Message::Ping(data) => match message_sender_for_thread.send(Message::Pong(data)) {
                            Ok(()) => (),
                            Err(e) => {
                                println!("Receive failed ping message: {:?}", e);
                                return;
                            }

                        },
                        _ => println!("Received: {:?}", message),
                    }
                }

            });

            if guard.join().is_err() {
                println!("Respawning receiver thread!");
            }
        }
    })
}

impl<'a> SlackStream<'a> {
    pub fn send_raw_message(&self, message: String) {
        let trimmed = message.trim();

        // let message = match trimmed {
        //     "/close" => {
        //         let _ = tx.send(Message::Close(None));
        //         break;
        //     }

        //     // so b"PING".to_vec is a Vec<u8> ?
        //     "/ping" => Message::Ping(b"PING".to_vec()),
        // };

        let message = Message::Text(trimmed.to_string());
        match self._message_sender.send(message) {
            Ok(()) => (),
            Err(e) => {
                println!("Main Loop: {:?}", e);
            }
        }
    }
}

