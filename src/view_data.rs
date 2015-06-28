use message::Message;
use channel::Channel;

// What the view renders at any given time
#[derive(Clone)]
pub struct ViewData {
    pub messages: Vec<Message>,
    pub channel: Channel,
    pub debug: Vec<String>
}

impl ViewData {
    pub fn new(channel: Channel) -> ViewData {
        ViewData { messages: vec![], channel: channel, debug: vec![]}
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }

    pub fn add_debug(&mut self, string: String) {
        self.debug.push(string);
    }
}
