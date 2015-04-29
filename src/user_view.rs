use std::io;
use std::thread;
use std::sync::mpsc;

use message::Message;
use channel::Channel;

pub struct UserView<'a> {
        _input_guard: thread::JoinGuard<'a, ()>,
        current_channel: Option<Channel>
}

pub fn start<'a>() -> (UserView<'a>, mpsc::Receiver<String>) {
    let (input_send, input_recv) = mpsc::channel::<String>();

    let view = UserView { _input_guard: spawn_input(input_send), current_channel: None };
    (view, input_recv)
}

impl<'a> UserView<'a> {
    pub fn print_message(&self, message: Message) -> Result<(), &'static str> {
       println!("{}", message);
       Ok(())
    }

    pub fn current_channel(&mut self, channel: Channel) -> Result<(), &'static str> {
       self.current_channel = Some(channel);
       Ok(())
    }
}

fn spawn_input<'b>(tx: mpsc::Sender<String>) -> thread::JoinGuard<'b, ()> {
   thread::scoped(move || {
      let mut stdin = io::stdin();

      loop {
         let mut input: String = "".to_string();
         print!(">> ");
         stdin.read_line(&mut input);
         tx.send(input).ok().expect("Could not send input");
      }
   })
}

#[cfg(test)]
mod test {
   use super::start;
   use channel::Channel;

   fn test_current_channel() {
      let (mut view, _) = start();
      assert!(view.current_channel.is_none());

      let chan = Channel { id: "xyz".to_string(), name: "General".to_string(), is_member: true, members: None };
      view.current_channel(chan).unwrap();
      assert_eq!(view.current_channel.unwrap().name, "General");
   }
}
