use serialize::json;
use std::fmt;

pub struct UserView;

// TODO Message needs to be normalized first
struct Message {
    event_type: String,
    user: String,
    text: String,
    ts: String,
    channel: String
}

pub fn new() -> UserView {
    UserView
}

impl UserView {
    pub fn print_message(&self, message: String) {
       let message = self.normalize(message);
       println!("{}", message);
    }

    // TODO clean up and make -> Result<Message, E>
    // Needs more validation, requirements depend on event
    // Working with empty strings because lazy
    // Event needs to be an enum
    fn normalize(&self, message: String) -> Message {
        let json = json::from_str(&message).unwrap();

        Message {
            event_type: value_from_json(&json, "type"),
            ts: value_from_json(&json, "ts"),
            user: value_from_json(&json, "user"),
            text: value_from_json(&json, "text"),
            channel: value_from_json(&json, "channel")
        }
    }
}

fn value_from_json(json: &json::Json, key: &str) -> String {
    json.find(key)
        .and_then(|json| json.as_string() )
        .unwrap_or("")
        .to_string()
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = match self.event_type.as_slice() {
            "message" => {
                // let channel = self.channel.clone().expect("Expected channel for message");
                format!("{ts} - #{channel} {user}: {text}",
                        ts=self.ts, channel=self.channel, user=self.user, text=self.text)
            },
            _ => format!("DEBUG: {ts} {type} {text}", type=self.event_type, ts=self.ts, text=self.text)
        };

        write!(f, "{}", text)
    }
}

// Meh...
// impl fmt::Display for Option<String> {
//     fn fmt(&self, f: &fmt::Formatter) -> Result<(), fmt::Error> {
//         write!(f, "{}", self.unwrap_or(""))
//     }
// }
