use std::io;
use std::thread;
use std::sync::{mpsc, Arc, Mutex};

use message::Message;
use channel::Channel;
use current_state::{self, CurrentState};
use dispatcher::{self, DispatchType, Subscribe, SubscribeHandle, DispatchMessage, Broadcast, BroadcastHandle};
use view::View;

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
      // output loop
      // Messages that generally result in something
      // being printed
      let state = self.current_state.clone();

      // For blocking on messages
      let rx = self.subscribe_rx.clone();

      // For broadcasting stuff
      let broadcast_tx = self.broadcast_tx.clone().expect("Expected broadcaster to be present in display controller");

      // For sending stuff to be printed to the screen
      let (print_tx, print_rx) = mpsc::channel::<String>();

      let vguard = thread::spawn(move || {
         let mut view = View::new();
         let onInput = Box::new(move |string: String| {
            let message = DispatchMessage { dispatch_type: DispatchType::UserInput, payload: string};
            broadcast_tx.send(message).unwrap();
         });
         view.init(onInput, print_rx);
      });

      let mguard = thread::spawn(move || {
         loop {
            let message = rx.lock().unwrap().recv().unwrap();
            match message.dispatch_type {
               DispatchType::RawIncomingMessage => {
                  let parsed = state.lock().unwrap().parse_incoming_message(&message.payload);
                  print_tx.send(format!("{}", parsed.unwrap())).ok().expect("could not send to view");
               },
               DispatchType::UserInput => {
                  print_tx.send(format!("User input: {}", &message.payload)).ok().expect("could not send to view");
               }
               _ => ()
            }
         }
      });

      // input
      // let iguard = thread::spawn(move || {
      //    let mut stdin = io::stdin();

      //    loop {
      //       let mut input: String = "".to_string();
      //       print!(">> ");
      //       stdin.read_line(&mut input);
      //       tx.send(input).ok().expect("Could not send input");
      //    }
      // });

      // self._output_guard = Some(oguard);
      // self._input_guard = Some(iguard);
   }

   fn print_message(&self, message: Message) -> Result<(), &'static str> {
      println!("{}", message);
      Ok(())
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

// #[cfg(test)]
// mod test {
//    use super::start;
//    use channel::Channel;

//    fn test_current_channel() {
//       let (mut view, _) = start();
//       assert!(view.current_channel.is_none());

//       let chan = Channel { id: "xyz".to_string(), name: "General".to_string(), is_member: true, members: None };
//       view.current_channel(chan).unwrap();
//       assert_eq!(view.current_channel.unwrap().name, "General");
//    }
// }
