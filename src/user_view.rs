use std::io;
use std::thread;
use std::sync::mpsc;

use message::Message;

pub struct UserView<'a> {
        _input_guard: thread::JoinGuard<'a, ()>,
}

pub fn start<'a>() -> (UserView<'a>, mpsc::Receiver<String>) {
    let (input_send, input_recv) = mpsc::channel::<String>();

    let view = UserView { _input_guard: spawn_input(input_send) };
    (view, input_recv)
}

impl<'a> UserView<'a> {
    pub fn incoming_message(&self, message: Message) -> Result<(), &'static str> {
       println!("{}", message);
       Ok(())
    }
}

fn spawn_input<'b>(tx: mpsc::Sender<String>) -> thread::JoinGuard<'b, ()> {
   thread::scoped(move || {
      let mut stdin = io::stdin();

      loop {
         let mut input: String = "".to_string();
         stdin.read_line(&mut input);
         tx.send(input).ok().expect("Could not send input");
      }
   })
}
