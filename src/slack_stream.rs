// Sets up a connections to the websocket stream
// I guess the only way to properly write unit tests for this
// is to make the websockets api generic... bad idea, not
// with unstable 3rd parties.

// std
use std::io::prelude::*;
use rustc_serialize::json::{self, Json, ToJson};
use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::{Arc, RwLock};
use std::collections::BTreeMap;

// extern
use hyper::client::Client as HttpClient;
use websocket::{Message, Client};
use websocket::client::sender::Sender as WssSender;
use websocket::Sender as WssSenderTrait;
use websocket::client::receiver::Receiver as WssReceiver;
use websocket::Receiver as WssReceiverTrait;
// use websocket::Sender as WssSender;
// use websocket::Receiver as WssReceiver;
use websocket::client::request::Url;
use websocket::stream::WebSocketStream;

use rdispatcher::{self, DispatchMessage};
use dispatch_type::DispatchType;

// TODO Use this
// const SLACK_RTM_START: &'static str = "https://slack.com/api/rtm.start?token={}";

// ... not really a stream
// More like a guard with extras
pub struct SlackStream {
    pub initial_state: Option<String>,

    // TODO Threadpool
    _receiver_guard: Option<thread::JoinHandle<()>>,
    _sender_guard: Option<thread::JoinHandle<()>>,

    _outgoing_sender: Option<Sender<Message>>,
    _incoming_sender: Option<Sender<rdispatcher::DispatchMessage<DispatchType>>>,
    sender_for_subscribe: Option<Sender<rdispatcher::DispatchMessage<DispatchType>>>
}

fn request_realtime_messaging(authtoken: &str) -> String {
    let mut client = HttpClient::new();
    let mut res = client.get(format!("https://slack.com/api/rtm.start?token={}", authtoken).as_str()).send().unwrap();
    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();

    body
}

fn spawn_send_loop<'a>(mut wss_sender: WssSender<WebSocketStream>, outgoing_receiver: Receiver<Message>) -> thread::JoinHandle<()> {
    let send_guard = thread::spawn(move || {
        loop {
            let message = match outgoing_receiver.recv() {
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

fn spawn_receive_loop<'a>(wss_receiver: WssReceiver<WebSocketStream>, outgoing_sender: Sender<Message>, incoming_sender: Sender<rdispatcher::DispatchMessage<DispatchType>>) -> thread::JoinHandle<()> {
    // Double thread to keep receiver alive
    // Previously crashed because ssl
    thread::spawn(move || {
        let arw_wss = Arc::new(RwLock::new(wss_receiver));
        loop {

            let arw_wss = arw_wss.clone();

            let outgoing_sender = outgoing_sender.clone();
            let incoming_sender = incoming_sender.clone();

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
                            let _ = match outgoing_sender.send(Message::Close(None)) {
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
                        Message::Ping(data) => match outgoing_sender.send(Message::Pong(data)) {
                            Ok(()) => (),
                            Err(e) => {
                                println!("Receive failed ping message: {:?}", e);
                                return;
                            }

                        },
                        Message::Text(text_message) => incoming_sender.send(as_dispatch_message(text_message)).unwrap(),

                        _ => panic!("Unknown message received from server!")
                    }
                }

            });

            if guard.join().is_err() {
                println!("Respawning receiver thread!");
            }
        }
    })
}

// #[derive(RustcEncodable)]
struct NormalizedSlackMessage {
    id: i32,
    channel: String,
    text: String,
    type_: String
}

impl ToJson for NormalizedSlackMessage {
    fn to_json(&self) -> Json {
        let mut d = BTreeMap::new();
        d.insert("id".to_string(), self.id.to_json());
        d.insert("channel".to_string(), self.channel.to_json());
        d.insert("text".to_string(), self.text.to_json());
        d.insert("type".to_string(), self.type_.to_json());
        Json::Object(d)
    }
}

fn spawn_normalize_loop(normalize_rx: Receiver<DispatchMessage<DispatchType>>, outgoing_tx: Sender<Message>) {
    thread::spawn(move || {
        let mut id = 0;
        loop {
            let dmessage = normalize_rx.recv().unwrap();
            let (channel_id, message) = json::decode(&dmessage.payload).unwrap();

            let json = NormalizedSlackMessage { id: id, channel: channel_id, text: message , type_: "message".to_string()}.to_json();
            id += 1;
            outgoing_tx.send(Message::Text(json.to_string()));
        }
    });
}

impl SlackStream {
    pub fn new() -> SlackStream {
        SlackStream {
            initial_state: None,
            _receiver_guard: None,
            _sender_guard: None,
            _outgoing_sender: None,
            _incoming_sender: None,
            sender_for_subscribe: None
        }
    }

    pub fn establish_stream(&mut self, authtoken: &str) {
        let initial_state = request_realtime_messaging(authtoken);
        // println!("{}", initial_state);
        let json = Json::from_str(&initial_state).unwrap();

        // As per example
        let wss_url = Url::parse(json.find("url").unwrap().as_string().unwrap()).unwrap();
        let request = Client::connect(wss_url).unwrap();
        let response = request.send().unwrap(); // send request and retrieve response
        response.validate().unwrap(); // validate the response (check wss frames, idk?)
        println!("Started wss connection");

        let (sender, receiver) = response.begin().split();

        // Outgoing messages are send via the sender, to the receiver, to websockets
        let (outgoing_sender, outgoing_receiver) = channel::<Message>();

        // Incoming messages are send via websockets to the sender, to the receiver, to the consumer
        // (i.e. UI)
        // let (incoming_sender, incoming_receiver) = channel::<rdispatcher::DispatchMessage>();

        let incoming_sender = self._incoming_sender.clone().expect("Expected incoming sender");

        let (normalize_tx, normalize_rx) = channel::<DispatchMessage<DispatchType>>();

        // Are the guards even needed?
        let send_guard = spawn_send_loop(sender, outgoing_receiver);
        let receiver_guard = spawn_receive_loop(receiver, outgoing_sender.clone(), incoming_sender);
        
        spawn_normalize_loop(normalize_rx, outgoing_sender.clone());
        println!("Spawned send and receive wss threads");

        self.initial_state = Some(initial_state);
        self._receiver_guard = Some(receiver_guard);
        self._sender_guard = Some(send_guard);
        self._outgoing_sender = Some(outgoing_sender);
        self.sender_for_subscribe = Some(normalize_tx);
    }

    pub fn send_raw_to_slack(&self, message: String) {
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
        match self._outgoing_sender {
            Some(ref sender) => match sender.send(message) {
                Ok(()) => (),
                Err(e) => {
                    println!("Main Loop: {:?}", e);
                }
            },
            None => panic!("No stream to send messages to")
        }
    }
}

impl rdispatcher::Broadcast<DispatchType> for SlackStream {
    fn broadcast_handle(&mut self) -> rdispatcher::BroadcastHandle<DispatchType> {
        let (incoming_sender, incoming_receiver) = channel::<rdispatcher::DispatchMessage<DispatchType>>();
        self._incoming_sender = Some(incoming_sender);
        incoming_receiver
    }
}

impl rdispatcher::Subscribe<DispatchType> for SlackStream {
    fn subscribe_handle(&self) -> rdispatcher::SubscribeHandle<DispatchType> {
        self.sender_for_subscribe.clone().expect("Expected slack stream to have outgoing sender")
    }
}

fn as_dispatch_message(payload: String) -> rdispatcher::DispatchMessage<DispatchType> {
    rdispatcher::DispatchMessage { dispatch_type: DispatchType::RawIncomingMessage, payload: payload }
}
