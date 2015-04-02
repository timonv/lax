use serialize::json::{self, Json, DecodeResult};

// Deprecated but RustcDecodable fails, wat
#[derive(Decodable)]
pub struct User {
    pub id: String,
    pub name: String,
    pub color: Option<String>
}

pub fn new_user_from_json(json: &str) -> DecodeResult<User> {
    json::decode::<User>(json)
}

#[cfg(test)]
mod test {
    use super::new_user_from_json;

    #[test]
    fn test_decode_from_json() {
        let json = "{
            \"id\": \"banana\",
            \"name\": \"Timon\",
            \"color\": \"#000000\"
        }";
        let user = new_user_from_json(json).unwrap();
        assert_eq!(user.id, "banana");
        assert_eq!(user.name, "Timon");
        assert_eq!(user.color.unwrap(), "#000000");
    }
}
