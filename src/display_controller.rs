use std::thread;
use std::sync::{mpsc, Arc, Mutex};
use rustc_serialize::json;

use input_parser;
use channel::Channel;
use current_state::{self, CurrentState};
use rdispatcher::{Subscribe, SubscribeHandle, DispatchMessage, Broadcast, BroadcastHandle};
use dispatch_type::DispatchType;
use view::View;
use view_data::ViewData;
use message::Message;

pub struct DisplayController {
   _view_guard: Option<thread::JoinHandle<()>>,
   _print_guard: Option<thread::JoinHandle<()>>,
   current_state: Arc<Mutex<CurrentState>>,
   subscribe_tx: mpsc::Sender<DispatchMessage<DispatchType>>,
   subscribe_rx: Arc<Mutex<mpsc::Receiver<DispatchMessage<DispatchType>>>>,
   broadcast_tx: Option<mpsc::Sender<DispatchMessage<DispatchType>>>
}

impl DisplayController {
   pub fn new(initial_state: &str) -> DisplayController {
      let initial_state = current_state::new_from_str(&initial_state);
      let (tx, rx) = mpsc::channel::<DispatchMessage<DispatchType>>();

      DisplayController {
         _view_guard: None,
         _print_guard: None,
         current_state: Arc::new(Mutex::new(initial_state)),
         subscribe_rx: Arc::new(Mutex::new(rx)),
         subscribe_tx: tx,
         broadcast_tx: None
      }
   }

   pub fn start(&mut self) {
      // For communicating from and to the view
      let (view_tx, view_rx) = mpsc::channel::<ViewData>();

      self.spawn_view_loop(view_rx);
      self.spawn_print_loop(view_tx);
   }

   fn spawn_view_loop(&mut self, view_rx: mpsc::Receiver<ViewData>) {
      let broadcast_tx = self.broadcast_tx.clone().expect("Expected broadcaster to be present in display controller");

      let guard = thread::spawn(move || {
         let mut view = View::new();
         let on_input = Box::new(move |string: String, channel_id: String| {
            let (payload, dtype) = input_parser::parse(string, channel_id);
            let message = DispatchMessage { payload: payload, dispatch_type: dtype };
            broadcast_tx.send(message).unwrap();
         });
         view.init(on_input, view_rx);
      });
      self._view_guard = Some(guard);
   }

   fn spawn_print_loop(&mut self, view_tx: mpsc::Sender<ViewData>) {
      let rx = self.subscribe_rx.clone();
      let state = self.current_state.clone();
      let default_channel = self.default_channel();

      let guard = thread::spawn(move || {
         let mut all_view_data: Vec<ViewData> = vec![];
         let mut current_view_data = ViewData::new(default_channel);
            
         loop {
            let message = rx.lock().unwrap().recv().unwrap();
            let locked_state = state.lock().unwrap();

            match message.dispatch_type {
               DispatchType::RawIncomingMessage   => handle_raw_incoming(&message.payload, &locked_state, &mut current_view_data, &mut all_view_data),
               DispatchType::ChangeCurrentChannel => handle_change_current_channel(&message.payload, &locked_state, &mut current_view_data, &mut all_view_data),
               DispatchType::ListChannels         => handle_list_channels(&message.payload, &locked_state, &mut current_view_data, &mut all_view_data),
               DispatchType::UserInput            => handle_user_input(&message.payload, &locked_state, &mut current_view_data)
               // _ => panic!("Got something I didn't expect: {:?}", message.payload)
            }
            current_view_data.update_unread(&all_view_data);
            view_tx.send(current_view_data.clone()).unwrap();
         }
      });
      self._print_guard = Some(guard);
   }

   fn default_channel(&self) -> Channel {
      self.current_state.lock().unwrap().default_channel()
         .expect("Could not find default channel")
         .clone()
   }
}

fn handle_raw_incoming(payload: &str, state: &CurrentState, current_view_data: &mut ViewData, all_view_data: &mut Vec<ViewData>) {
   match state.parse_incoming_message(payload) {
      Ok(parsed) => {
         match parsed.channel.as_ref() {
            Some(channel) if channel == &current_view_data.channel => {
               current_view_data.add_message(parsed.clone())
            },
            Some(channel) => {
               for data in all_view_data.iter_mut() {
                  if &data.channel == channel {
                     data.add_message(parsed.clone());
                     data.has_unread = true;
                     break;
                  }
               }

            },
            None => debug!("{}", parsed)
         }
      },
      Err(err) => debug!("Failed to parse message {} and gave err: {}",payload, err)
   };
}

fn handle_change_current_channel(payload: &str, state: &CurrentState, mut current_view_data: &mut ViewData, all_view_data: &mut Vec<ViewData>) {
   match state.name_to_channel(payload) {
      Some(channel) => {
         all_view_data.push(current_view_data.clone());
         match all_view_data.iter().position(|d| &d.channel == channel) {
            Some(idx) => {
               *current_view_data = all_view_data.remove(idx);
               current_view_data.has_unread = false;
            },
            None => {
               *current_view_data = ViewData::new(channel.clone());

            }
         }
         current_view_data.add_debug(format!("Changed channel to: {}", channel.name))
      },
      None => {
         current_view_data.add_debug(format!("Channel not found: {}", payload))
      }
   }
}

fn handle_list_channels(payload: &str, state: &CurrentState, current_view_data: &mut ViewData, all_view_data: &mut Vec<ViewData>) {
   let channel_names = state.channel_names();
   current_view_data.add_debug(format!("Channels: {}", channel_names.connect(", ")));
}

fn handle_user_input(payload: &str, state: &CurrentState, current_view_data: &mut ViewData) {
   let me = state.me.clone();
   // Only interested in message, not channel
   let (_, payload): (String, String) = json::decode(&payload).unwrap();
   let message = Message {
      ts: None,
      text: Some(payload),
      user: Some(me),
      channel: None,
      event_type: Some("message".to_string()),
      user_id: None,
      channel_id: None
   };
   current_view_data.add_message(message);
}

impl Subscribe<DispatchType> for DisplayController {
   fn subscribe_handle(&self) -> SubscribeHandle<DispatchType> {
      self.subscribe_tx.clone()
   }
}

impl Broadcast<DispatchType> for DisplayController {
    fn broadcast_handle(&mut self) -> BroadcastHandle<DispatchType> {
        let (tx, rx) = mpsc::channel::<DispatchMessage<DispatchType>>();
        self.broadcast_tx = Some(tx);
        rx
    }
}
