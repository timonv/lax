use rustc_serialize::json::{self, DecoderError};
use rustc_serialize::{Decodable, Decoder};
use std::fmt;

use user::User;
use channel::Channel;

#[derive(Debug, Clone)]
pub struct Message {
    pub ts: Option<String>,
    pub text: Option<String>,
    pub user: Option<User>,
    pub channel: Option<Channel>,
    pub event_type: Option<String>,// TODO Enum
    pub user_id: Option<String>,
    pub channel_id: Option<String>,
    // payload: String

}

pub fn new_from_str(payload: &str) -> Result<Message, DecoderError> {
    json::decode::<Message>(payload)
}

impl Decodable for Message {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Message, D::Error> {
        decoder.read_struct("root", 0, |decoder| {
            Ok(Message {
                event_type: try!(decoder.read_struct_field("type", 0, |decoder| Decodable::decode(decoder))),
                user_id: try!(decoder.read_struct_field("user", 0, |decoder| Decodable::decode(decoder))),
                text: try!(decoder.read_struct_field("text", 0, |decoder| Decodable::decode(decoder))),
                ts: try!(decoder.read_struct_field("ts", 0, |decoder| Decodable::decode(decoder))),
                channel_id: try!(decoder.read_struct_field("channel", 0, |decoder| Decodable::decode(decoder))),
                channel: None,
                user: None
            })
        })
    }
}

// Temporary method for debugging purposes.
// Rendering is not the job of the message.
impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO This is weird.
        let formatted = match self.event_type.as_ref().unwrap_or(&"".to_string()).as_ref() {
            "message" => self.fmt_as_message(),
            _         => self.fmt_as_debug(),
        };
        write!(f, "{}", formatted)
    }
}

impl Message {
    fn fmt_as_message(&self) -> String {
        let channel: &Channel = self.channel.as_ref().unwrap();
        let user: &User = self.user.as_ref().unwrap();
        format!("{channel} - {user}: {message}", channel=channel.name, user=user.name, message=self.text.as_ref().unwrap() )
    }

    fn fmt_as_debug(&self) -> String {
        format!("DEBUG: {:?}", &self)
    }
}

#[cfg(test)]
mod test {
    use super::new_from_str;

    #[test]
    fn test_decode_from_json() {
        let json = "{
            \"type\": \"message\",
            \"user\": \"Timon\",
            \"text\": \"Bananas!\",
            \"ts\": \"today\",
            \"channel\": \"banter\"
        }";
        let message = new_from_str(json).unwrap();
        assert_eq!(message.event_type.unwrap(), "message");
        assert_eq!(message.user_id.unwrap(), "Timon");
        assert_eq!(message.channel_id.unwrap(), "banter");
        assert_eq!(message.text.unwrap(), "Bananas!");
        assert_eq!(message.ts.unwrap(), "today");
    }
}

