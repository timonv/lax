// Yolo code, badly needs cleanup
use rustc_serialize::json::{self, Json};

use channel::{self, Channel};
use user::{self, User};
use message::Message;

pub struct CurrentState {
    me: User,
    channels: Vec<Channel>,
    users: Vec<User>
}

pub fn new_from_str(json: &str) -> CurrentState {
    CurrentState {
        me: extract_me(&json),
        channels: extract_channels(&json),
        users: extract_users(&json),
    }
}


fn extract_me(json: &str) -> User {
    // Hideous, maybe better to just manually implement decodable?
    // also use try! instead.
    let json = Json::from_str(json).unwrap();
    let val = json.find("self").unwrap();
    user::new_from_str(json::encode(&val.as_object()).unwrap().as_ref()).unwrap()
}

fn extract_users(json: &str) -> Vec<User> {
    // Hideous, maybe better to just manually implement decodable?
    // also use try! instead.
    let json = Json::from_str(json).unwrap();
    json.find("users").unwrap().as_array().unwrap().iter().map(|user| {
        user::new_from_str(json::encode(user.as_object().unwrap()).unwrap().as_ref()).unwrap()
    }).collect()
}

fn extract_channels(json: &str) -> Vec<Channel> {
    // Hideous, maybe better to just manually implement decodable?
    // also use try! instead.
    let json = Json::from_str(json).unwrap();
    json.find("channels").unwrap().as_array().unwrap().iter().map(|channel| {
        channel::new_from_str(json::encode(channel.as_object().unwrap()).unwrap().as_ref()).unwrap()
    }).collect()
}

impl CurrentState {
    pub fn parse_incoming_message(&self, raw: &str) -> json::DecodeResult<Message> {
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

    #[test]
    fn test_new_current_state_from_str() {
        let state = new_from_str(&generate_json());
        assert_eq!(state.me.name, "bobby");
        assert_eq!(state.users[0].name, "Matijs");
        assert_eq!(state.channels[0].name, "General");
    }

    #[test]
    fn test_id_to_user() {
        let state = new_from_str(&generate_json());
        let user = state.id_to_user("xyz");
        assert_eq!(user.unwrap().name, "Matijs");
    }

    #[test]
    fn test_id_to_channel() {
        let state = new_from_str(&generate_json());
        let channel = state.id_to_channel("zyx");
        assert_eq!(channel.unwrap().name, "General");
    }

    #[test]
    fn test_parse_incoming_message() {
        let state = new_from_str(&generate_json());
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
        let state = new_from_str(&generate_json());
        let channel = state.name_to_channel("General").expect("Could not find channel");
        assert_eq!(channel.name, "General");
    }

    #[test]
    fn test_channel_names() {
        let state = new_from_str(&generate_json());
        assert_eq!(state.channel_names(), vec!["General", "Dev"]);
    }

    fn generate_json() -> String {
        "{
            \"ok\": true,
            \"url\": \"wss://ms9.slack-msgs.com/websocket/7I5yBpcvk\",
            \"self\": {
                \"id\": \"U023BECGF\",
                \"name\": \"bobby\",
                \"prefs\": {
                },
                \"created\": 1402463766,
                \"manual_presence\": \"active\"
            },
            \"team\": {
                \"id\": \"T024BE7LD\",
                \"name\": \"Example Team\",
                \"email_domain\": \"\",
                \"domain\": \"example\",
                \"msg_edit_window_mins\": -1,
                \"over_storage_limit\": false,
                \"prefs\": {
                },
                \"plan\": \"std\"
            },
            \"users\": [
                {
                    \"id\": \"xyz\",
                    \"name\": \"Matijs\"
                }
            ],
            \"channels\": [
                {
                    \"id\": \"zyx\",
                    \"name\": \"General\",
                    \"members\": [],
                    \"is_member\": false,
                    \"is_general\": false
                },
                {
                    \"id\": \"xyz\",
                    \"name\": \"Dev\",
                    \"members\": [],
                    \"is_member\": false,
                    \"is_general\": false
                }

            ],
            \"groups\": [ ],
            \"ims\": [ ],
            \"bots\": [ ]
        }".to_string()
    }
}
