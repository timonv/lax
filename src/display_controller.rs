use std::thread;
use std::sync::{mpsc, Arc, Mutex};

use input_parser;
use message::Message;
use channel::Channel;
use current_state::{self, CurrentState};
use dispatcher::{DispatchType, Subscribe, SubscribeHandle, DispatchMessage, Broadcast, BroadcastHandle};
use view::View;
use view_data::ViewData;

pub struct DisplayController {
   _input_guard: Option<thread::JoinHandle<()>>,
   _output_guard: Option<thread::JoinHandle<()>>,
   current_state: Arc<Mutex<CurrentState>>,
   subscribe_tx: mpsc::Sender<DispatchMessage>,
   subscribe_rx: Arc<Mutex<mpsc::Receiver<DispatchMessage>>>,
   broadcast_tx: Option<mpsc::Sender<DispatchMessage>>
}

impl DisplayController {
   pub fn new(initial_state: &str) -> DisplayController {
      let initial_state = current_state::new_from_str(&initial_state);
      let (tx, rx) = mpsc::channel::<DispatchMessage>();

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
         let on_input = Box::new(move |string: String| {
            let (payload, dtype) = input_parser::parse(string);
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
         let mut view_data = ViewData::new(default_channel.clone());
         loop {
            let message = rx.lock().unwrap().recv().unwrap();
            match message.dispatch_type {
               DispatchType::RawIncomingMessage => {
                  let parsed = state.lock().unwrap().parse_incoming_message(&message.payload).unwrap();
                  // Innefficient clone that saves a clone on the parsed, lesser evil
                  let channel = parsed.channel.clone();
                  // Would be nice if rust could figure out parsed was done and movable after the if
                  if channel.is_some() && channel.unwrap() == view_data.channel {
                     view_data.add_message(parsed)
                  } else {
                     view_data.add_debug(format!("{}", parsed))
                  }
               },
               DispatchType::UserInput => {
                  view_data.add_debug(format!("User input: {}", &message.payload))
               }
               _ => ()
            }
            view_tx.send(view_data.clone()).unwrap();
         }
      });
   }

   fn default_channel(&self) -> Channel {
      self.current_state.lock().unwrap().default_channel()
         .expect("Could not find default channel")
         .clone()
   }
}

impl Subscribe for DisplayController {
   fn subscribe_handle(&self) -> SubscribeHandle {
      self.subscribe_tx.clone()
   }
}

impl Broadcast for DisplayController {
    fn broadcast_handle(&mut self) -> BroadcastHandle {
        let (tx, rx) = mpsc::channel::<DispatchMessage>();
        self.broadcast_tx = Some(tx);
        rx
    }
}
