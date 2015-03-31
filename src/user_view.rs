// use serialize::json;
// use std::fmt;

// pub struct UserView {
//     state: CurrentState
// }

// pub fn new() -> UserView {
//     UserView {
//         state: new_empty_state()
//     }
// }

// fn new_empty_state() -> CurrentState {
//     CurrentState {
//         me: None,
//         channels: None,
//         current_channel: None
//     }
// }

// impl UserView {
//     pub fn print_message(&self, message: String) {
//        let message = self.normalize_from_json(message).unwrap();
//        println!("{}", message);
//     }

//     pub fn update_state(&mut self, json: Json) {
//         self.state = state.update(json);
//     }

//     fn normalize_from_json(&self, message: String) -> Result<Message, E> {
//         let message: Message = json::decode(&message);
//         message
//     }
// }

// impl fmt::Display for Message {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         let text = match self.event_type.as_slice() {
//             "message" => {
//                 // let channel = self.channel.clone().expect("Expected channel for message");
//                 format!("{ts} - #{channel} {user}: {text}",
//                         ts=self.ts, channel=self.channel, user=self.user, text=self.text)
//             },
//             _ => format!("DEBUG: {ts} {type} {text}", type=self.event_type, ts=self.ts, text=self.text)
//         };

//         write!(f, "{}", text)
//     }
// }

// Meh...
// impl fmt::Display for Option<String> {
//     fn fmt(&self, f: &fmt::Formatter) -> Result<(), fmt::Error> {
//         write!(f, "{}", self.unwrap_or(""))
//     }
// }
