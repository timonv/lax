use serialize::json::{self, Json, DecodeResult};

// Deprecated but RustcDecodable fails, wat
#[derive(Decodable)]
pub struct Channel {
    pub id: String,
    pub name: String,
    pub members: Vec<String>,
    pub is_member: bool
}

pub fn new_channel_from_str(json: &str) -> DecodeResult<Channel> {
    json::decode::<Channel>(json)
}

#[cfg(test)]
mod test {
    use super::new_channel_from_str;

    #[test]
    fn test_decode_from_str() {
        let json = "{
            \"id\": \"banana\",
            \"name\": \"banter\",
            \"members\": [\"Timon\"],
            \"is_member\": false
        }";
        let channel = new_channel_from_str(json).unwrap();
        assert_eq!(channel.id, "banana");
        assert_eq!(channel.name, "banter");
        assert_eq!(channel.members, vec!["Timon"]);
        assert_eq!(channel.is_member, false);
    }
}

