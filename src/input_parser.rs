use dispatch_type::DispatchType;
use rustc_serialize::json;


// Return tuple for less dependencies
pub fn parse(payload: String, channel_id: String) -> (String, DispatchType) {
    let vec: Vec<&str> = payload.split(' ').collect();
    match vec[0] {
        "/channel" => {
            if vec.len() > 1 {
                (vec[1].to_string(), DispatchType::ChangeCurrentChannel)
            } else {
                ("".to_string(), DispatchType::ListChannels)
            }
        },
        _ => {
            // Hack as dispatcher needs a more complex type, but complex enums (i.e.
            // UserInput(channel) are currently not supported by the dispatcher.
            let payload = json::encode(&vec![channel_id, payload.clone()]);
            (payload.clone().unwrap(), DispatchType::UserInput)
        } 
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use dispatch_type::DispatchType;
    use rustc_serialize::json;

    #[test]
    fn test_normal_input() {
        let (payload, dispatch_type) = parse("Hello world!".to_string(), "pong".to_string());
        let parsed: Vec<String> = json::decode(&payload).unwrap();
        assert_eq!(parsed[0], "pong".to_string());
        assert_eq!(parsed[1], "Hello world!".to_string());
        assert_eq!(dispatch_type, DispatchType::UserInput);
    }

    #[test]
    fn test_change_channel_command() {
        let (payload, dispatch_type) = parse("/channel banter".to_string(), "pong".to_string());
        assert_eq!(payload, "banter".to_string());
        assert_eq!(dispatch_type, DispatchType::ChangeCurrentChannel);
    }

    #[test]
    fn test_list_channel_command() {
        let (payload, dispatch_type) = parse("/channel".to_string(), "pong".to_string());
        assert_eq!(payload, "".to_string());
        assert_eq!(dispatch_type, DispatchType::ListChannels);
    }
}
