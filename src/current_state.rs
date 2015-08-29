// Yolo code, badly needs cleanup
use rustc_serialize::json::{self, Json};

use channel::{self, Channel};
use user::{self, User};
use message::Message;

pub type ParseResult<T> = Result<T, String>;

pub struct CurrentState {
    pub me: User,
    channels: Vec<Channel>,
    users: Vec<User>
}

pub fn new_from_str(json: &str) -> ParseResult<CurrentState> {
    debug!("NEW CURRENT STATE: {}", json);
    Ok(CurrentState {
        me: try!(extract_me(&json)),
        channels: try!(extract_channels(&json)),
        users: try!(extract_users(&json)),
    })
}

//Experiment with different styles of unwrap cleanup
fn extract_me(json: &str) -> ParseResult<User> {
    let json = try!(Json::from_str(json).map_err(|e| e.to_string()));
    json.find("self").ok_or("[current_state] Could not find self".to_string())
        .and_then( |json_self| json::encode(&json_self.as_object()).map_err(|e| e.to_string()))
        .and_then( |encoded| user::new_from_str(&encoded).map_err(|e| e.to_string()))
}

fn extract_users(json: &str) -> ParseResult<Vec<User>> {
    let json = try!(Json::from_str(json).map_err(|e| e.to_string()));
    Ok(json.find("users")
        .expect("Could not find users")
        .as_array()
        .expect("Expected array")
        .iter()
        .map(|user| {
            let encoded = json::encode(user.as_object().unwrap()).unwrap();
            user::new_from_str(&encoded).unwrap()
        }).collect())
}

fn extract_channels(json: &str) -> ParseResult<Vec<Channel>> {
    let json = Json::from_str(json).unwrap();
    Ok(json.find("channels")
       .expect("Could not find channels")
       .as_array()
       .unwrap()
       .iter()
       .map(|channel| {
           let encoded = json::encode(channel.as_object().unwrap()).unwrap();
           channel::new_from_str(&encoded).unwrap()
    }).collect())
}

impl CurrentState {
    pub fn parse_incoming_message(&self, raw: &str) -> json::DecodeResult<Message> {
        debug!("Incoming message: {}", raw);
        let mut message = try!(Message::new_from_str(&raw));
        match message.channel_id {
            // I'm not sure I fully understand why the ref is needed.
            // The value is borrowed, but isn't it returned after going out of scope?
            Some(ref id) => message.channel = self.id_to_channel(id).map(|chan| chan.clone()),
            None => ()
        }

        match message.user_id {
            Some(ref id) => message.user = self.id_to_user(id).map(|user| user.clone()),
            None => ()
        }

        Ok(message)
    }

    pub fn name_to_channel(&self, name: &str) -> Option<&Channel> {
        self.channels.iter().find(|channel| channel.name == name)
    }

    pub fn default_channel(&self) -> Option<&Channel> {
        self.channels.iter().find(|channel| channel.is_general == true)
    }

    pub fn channel_names(&self) -> Vec<String> {
        self.channels.iter().map(|ref channel| channel.name.clone() ).collect()
    }


    fn id_to_user(&self, id: &str) -> Option<&User> {
        self.users.iter().find(|user| user.id == id)
    }
    
    fn id_to_channel(&self, id: &str) -> Option<&Channel> {
        self.channels.iter().find(|channel| channel.id == id)
    }

}

#[cfg(test)]
mod test {
    use super::new_from_str;
    use test_helpers::*;

    #[test]
    fn test_new_current_state_from_str() {
        let state = setup_current_state();
        assert_eq!(state.me.name, "bobby");
        assert_eq!(state.users[0].name, "Matijs");
        assert_eq!(state.channels[0].name, "General");
    }

    #[test]
    fn test_id_to_user() {
        let state = setup_current_state();
        let user = state.id_to_user("xyz");
        assert_eq!(user.unwrap().name, "Matijs");
    }

    #[test]
    fn test_id_to_channel() {
        let state = setup_current_state();
        let channel = state.id_to_channel("zyx");
        assert_eq!(channel.unwrap().name, "General");
    }

    #[test]
    fn test_parse_incoming_message() {
        let state = setup_current_state();
        let json = "{
            \"type\": \"message\",
            \"user\": \"xyz\",
            \"text\": \"Bananas!\",
            \"ts\": \"today\",
            \"channel\": \"zyx\"
        }";

        let message = state.parse_incoming_message(&json).unwrap();
        assert_eq!(message.channel.unwrap().name, "General");
        assert_eq!(message.user.unwrap().name, "Matijs");
    }

    #[test]
    fn test_name_to_channel() {
        let state = setup_current_state();
        let channel = state.name_to_channel("General").expect("Could not find channel");
        assert_eq!(channel.name, "General");
    }

    #[test]
    fn test_channel_names() {
        let state = setup_current_state();
        assert_eq!(state.channel_names(), vec!["General", "Dev"]);
    }

}
