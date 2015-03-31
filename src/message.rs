use serialize::json::{self, Json, DecodeResult};

// Deprecated but RustcDecodable fails, wat
#[derive(Decodable)]
struct Message {
    event_type: String,
    user: String,
    text: String,
    ts: String,
    channel: String
}

fn new_message_from_json(json: &str) -> DecodeResult<Message> {
    json::decode::<Message>(json)
}

#[cfg(test)]
mod test {
    use super::new_message_from_json;

    #[test]
    fn test_decode_from_json() {
        let json = "{
            \"event_type\": \"message\",
            \"user\": \"Timon\",
            \"text\": \"Bananas!\",
            \"ts\": \"today\",
            \"channel\": \"banter\"
        }";
        let message = new_message_from_json(json).unwrap();
        assert_eq!(message.event_type, "message");
        assert_eq!(message.user, "Timon");
        assert_eq!(message.text, "Bananas!");
        assert_eq!(message.ts, "today");
        assert_eq!(message.channel, "banter");
    }
}

