use rustc_serialize::json::{self, DecoderError};

#[derive(RustcDecodable, Clone, Debug)]
pub struct User {
    pub id: String,
    pub name: String,
    pub color: Option<String>
}

pub fn new_from_str(json: &str) -> Result<User, DecoderError> {
    json::decode::<User>(json)
}

#[cfg(test)]
mod test {
    use super::new_from_str;

    #[test]
    fn test_decode_from_str() {
        let json = "{
            \"id\": \"banana\",
            \"name\": \"Timon\",
            \"color\": \"#000000\"
        }";
        let user = new_from_str(json).unwrap();
        assert_eq!(user.id, "banana");
        assert_eq!(user.name, "Timon");
        assert_eq!(user.color.unwrap(), "#000000");
    }
}

