use std::io;
use std::thread;
use std::sync::{mpsc, Arc, Mutex};

use message::Message;
use channel::Channel;
use current_state::{self, CurrentState};
use dispatcher::{self, DispatchType, Subscribe, SubscribeHandle, DispatchMessage};

// Not really a view, rename to something sensible
pub struct UserView {
   _input_guard: Option<thread::JoinHandle<()>>,
   _output_guard: Option<thread::JoinHandle<()>>,
   current_state: Arc<Mutex<CurrentState>>,
   subscribe_tx: mpsc::Sender<DispatchMessage>,
   subscribe_rx: Arc<Mutex<mpsc::Receiver<DispatchMessage>>>
}

impl UserView {
   pub fn new(initial_state: &str) -> UserView {
      let initial_state = current_state::new_from_str(&initial_state);
      let (tx, rx) = mpsc::channel::<DispatchMessage>();

      UserView {
         _input_guard: None,
         _output_guard: None,
         current_state: Arc::new(Mutex::new(initial_state)),
         subscribe_rx: Arc::new(Mutex::new(rx)),
         subscribe_tx: tx
      }
   }

   pub fn start(&mut self) {
      // output loop
      // Messages that generally result in something
      // being printed
      let state = self.current_state.clone();
      let rx = self.subscribe_rx.clone();

      let oguard = thread::spawn(move || {
         loop {
            let message = rx.lock().unwrap().recv().unwrap();
            match message.dispatch_type {
               DispatchType::RawIncomingMessage => {
                  let parsed = state.lock().unwrap().parse_incoming_message(&message.payload);
                  println!("{}", parsed.unwrap());
               },
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

      self._output_guard = Some(oguard);
      // self._input_guard = Some(iguard);
   }

   fn print_message(&self, message: Message) -> Result<(), &'static str> {
      println!("{}", message);
      Ok(())
   }
}

impl Subscribe for UserView {
   fn subscribe_handle(&self) -> SubscribeHandle {
      self.subscribe_tx.clone()
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
