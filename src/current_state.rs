use serialize::json;

use channel::{Channel, new_channel_from_json};
use user::{User, new_user_from_json};

pub struct CurrentState {
    me: User,
    channels: Vec<Channel>,
    users: Vec<User>
    // current_channel: Channel
}

pub fn new_current_state_from_json(json: &str) -> CurrentState {
    CurrentState {
        me: extract_me(&json),
        channels: vec![],
        users: extract_users(&json)
    }
}

fn extract_me(json: &str) -> User {
    // Hideous, maybe better to just manually implement decodable?
    // also use try! instead.
    let json = json::from_str(json).unwrap();
    let val = json.find("self").unwrap();
    new_user_from_json(json::encode(&val.as_object()).unwrap().as_slice()).unwrap()
}

fn extract_users(json: &str) -> Vec<User> {
    // Hideous, maybe better to just manually implement decodable?
    // also use try! instead.
    let json = json::from_str(json).unwrap();
    json.find("users").unwrap().as_array().unwrap().iter().map(|user| {
        new_user_from_json(json::encode(user.as_object().unwrap()).unwrap().as_slice()).unwrap()
    }).collect()
}

impl CurrentState {
    pub fn user_id_to_user(&self, id: &str) -> Option<&User> {
        self.users.iter().find(|user| user.id == id)
    }
}

#[cfg(test)]
mod test {
    use super::new_current_state_from_json;

    #[test]
    fn test_new_current_state_from_json() {
        let state = new_current_state_from_json(generate_json().as_slice());
        assert_eq!(state.me.name, "bobby");
    }

    #[test]
    fn test_new_current_state_with_user() {
        let state = new_current_state_from_json(generate_json_with_users().as_slice());
        assert_eq!(state.users[0].name, "Matijs");
    }

    #[test]
    fn test_id_to_user() {
        let state = new_current_state_from_json(generate_json_with_users().as_slice());
        let user = state.user_id_to_user("xyz");
        assert_eq!(user.unwrap().name, "Matijs");
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
            \"users\": [ ],
            \"channels\": [ ],
            \"groups\": [ ],
            \"ims\": [ ],
            \"bots\": [ ]
        }".to_string()
    }

    fn generate_json_with_users() -> String {
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
            \"channels\": [ ],
            \"groups\": [ ],
            \"ims\": [ ],
            \"bots\": [ ]
        }".to_string()
    }
}
