use serialize::json;

use channel::{Channel, new_channel_from_json};
use user::{User, new_user_from_json};

pub struct CurrentState {
    me: User,
    channels: Vec<Channel>
    // current_channel: Channel
}

pub fn new_current_state_from_json(json: &str) -> CurrentState {
    CurrentState {
        me: extract_me(&json),
        channels: vec![]
    }
}

fn extract_me(json: &str) -> User {
    // Hideous, maybe better to just manually implement decodable?
    // also use try! instead.
    let json = json::from_str(json).unwrap();
    let val = json.find("self").unwrap();
    new_user_from_json(json::encode(&val.as_object()).unwrap().as_slice()).unwrap()
}

#[cfg(test)]
mod test {
    use super::new_current_state_from_json;

    #[test]
    fn test_new_current_state_from_json() {
        let state = new_current_state_from_json(generate_json().as_slice());
        assert_eq!(state.me.name, "bobby")
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
}
