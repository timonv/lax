use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver, self};

use message::Message;

pub struct UserView<'a> {
    _incoming_sender: Sender<Message>,
    _view_guard: thread::JoinGuard<'a, ()>
}

pub fn start<'a>() -> UserView<'a> {
    let (tx, rx) = channel::<Message>();
    let view_guard = spawn_view(rx);

    UserView {
        _incoming_sender: tx,
        _view_guard: view_guard
    }
}

impl<'a> UserView<'a> {
    pub fn incoming_message(&self, message: Message) -> Result<(), mpsc::SendError<Message>> {
       self._incoming_sender.send(message)
    }
}

fn spawn_view<'b>(rx: Receiver<Message>) -> thread::JoinGuard<'b, ()> {
   thread::scoped(move || {
       loop {
           let message = rx.recv().ok().expect("Failed to print message");
           println!("{}", message);
       }
   })
}

