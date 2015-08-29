use current_state::{CurrentState, new_from_str};

pub fn setup_current_state() -> CurrentState {
    new_from_str(&current_state_json()).unwrap()
}

fn current_state_json() -> String {
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
