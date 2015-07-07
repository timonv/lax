use std::thread;
use std::sync::{mpsc, Arc, Mutex};

use input_parser;
use channel::Channel;
use current_state::{self, CurrentState};
use rdispatcher::{Subscribe, SubscribeHandle, DispatchMessage, Broadcast, BroadcastHandle};
use dispatch_type::DispatchType;
use view::View;
use view_data::ViewData;

pub struct DisplayController {
   _input_guard: Option<thread::JoinHandle<()>>,
   _output_guard: Option<thread::JoinHandle<()>>,
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
         _input_guard: None,
         _output_guard: None,
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

   fn spawn_view_loop(&self, view_rx: mpsc::Receiver<ViewData>) {
      let broadcast_tx = self.broadcast_tx.clone().expect("Expected broadcaster to be present in display controller");

      thread::spawn(move || {
         let mut view = View::new();
         let on_input = Box::new(move |string: String, channel_id: String| {
            let (payload, dtype) = input_parser::parse(string, channel_id);
            let message = DispatchMessage { payload: payload, dispatch_type: dtype };
            broadcast_tx.send(message).unwrap();
         });
         view.init(on_input, view_rx);
      });
   }

   fn spawn_print_loop(&self, view_tx: mpsc::Sender<ViewData>) {
      let rx = self.subscribe_rx.clone();
      let state = self.current_state.clone();
      let default_channel = self.default_channel();

      thread::spawn(move || {
         let mut all_view_data: Vec<ViewData> = vec![];
         let mut current_view_data = ViewData::new(default_channel.clone());
            
         loop {
            let message = rx.lock().unwrap().recv().unwrap();
            match message.dispatch_type {
               DispatchType::RawIncomingMessage => {
                  let parsed = state.lock().unwrap().parse_incoming_message(&message.payload).unwrap();

                  match parsed.channel.as_ref() {
                     Some(channel) if channel == &current_view_data.channel => {
                        current_view_data.add_message(parsed.clone())
                     },
                     Some(channel) => {
                        for data in all_view_data.iter_mut() {
                           if &data.channel == channel {
                              data.add_message(parsed.clone());
                              break;
                           }
                        }

                     },
                     None => current_view_data.add_debug(format!("{}", parsed))
                  }
               },
               // DispatchType::UserInput => {
               //    current_view_data.add_debug(format!("User input: {}", &message.payload))
               // },
               DispatchType::ChangeCurrentChannel => {
                  match state.lock().unwrap().name_to_channel(&message.payload) {
                     Some(channel) => {
                        all_view_data.push(current_view_data.clone());
                        match all_view_data.iter().position(|d| &d.channel == channel) {
                           Some(idx) => {
                              current_view_data = all_view_data.remove(idx);
                           },
                           None => {
                              current_view_data = ViewData::new(channel.clone());

                           }
                        }
                        current_view_data.add_debug(format!("Changed channel to: {}", channel.name))
                     },
                     None => {
                        current_view_data.add_debug(format!("Channel not found: {}", &message.payload))
                     }
                  }
               },
               DispatchType::ListChannels => {
                  let channel_names = state.lock().unwrap().channel_names();
                  current_view_data.add_debug(format!("Channels: {}", channel_names.connect(", ")));
               },
               _ => ()
            }
            view_tx.send(current_view_data.clone()).unwrap();
         }
      });
   }

   fn default_channel(&self) -> Channel {
      self.current_state.lock().unwrap().default_channel()
         .expect("Could not find default channel")
         .clone()
   }
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
