use dispatch_type::DispatchType;

// Return tuple for less dependencies
pub fn parse(payload: String) -> (String, DispatchType) {
    let vec: Vec<&str> = payload.split(' ').collect();
    match vec[0] {
        "/channel" => {
            if vec.len() > 1 {
                (vec[1].to_string(), DispatchType::ChangeCurrentChannel)
            } else {
                ("".to_string(), DispatchType::ListChannels)
            }
        },
        _ => (payload.clone(), DispatchType::UserInput)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use dispatch_type::DispatchType;

    #[test]
    fn test_normal_input() {
        let (payload, dispatch_type) = parse("Hello world!".to_string());
        assert_eq!(payload, "Hello world!".to_string());
        assert_eq!(dispatch_type, DispatchType::UserInput);
    }

    #[test]
    fn test_change_channel_command() {
        let (payload, dispatch_type) = parse("/channel banter".to_string());
        assert_eq!(payload, "banter".to_string());
        assert_eq!(dispatch_type, DispatchType::ChangeCurrentChannel);
    }

    #[test]
    fn test_list_channel_command() {
        let (payload, dispatch_type) = parse("/channel".to_string());
        assert_eq!(payload, "".to_string());
        assert_eq!(dispatch_type, DispatchType::ListChannels);
    }
}
