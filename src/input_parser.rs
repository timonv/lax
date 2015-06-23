use dispatcher::DispatchType;

// Return tuple for less dependencies
pub fn parse(payload: String) -> (String, DispatchType) {
    let vec: Vec<&str> = payload.split(' ').collect();
    match vec[0] {
        "/channel" => (vec[1].to_string(), DispatchType::ChangeCurrentChannel),
        _ => (payload.clone(), DispatchType::UserInput)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use dispatcher::DispatchType;

    fn test_normal_input() {
        let (payload, dispatch_type) = parse("Hello world!".to_string());
        assert_eq!(payload, "Hello world!".to_string());
        assert_eq!(dispatch_type, DispatchType::UserInput);
    }

    fn test_change_channel_command() {
        let (payload, dispatch_type) = parse("/channel banter".to_string());
        assert_eq!(payload, "banter".to_string());
        assert_eq!(dispatch_type, DispatchType::ChangeCurrentChannel);
    }
}
