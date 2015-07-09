use message::Message;
use channel::Channel;

// What the view renders at any given time
#[derive(Clone)]
pub struct ViewData {
    pub messages: Vec<Message>,
    pub channel: Channel,
    pub debug: Vec<String>,
    pub has_unread: bool,
    pub unread_channels: Vec<Channel>
}

impl ViewData {
    pub fn new(channel: Channel) -> ViewData {
        ViewData { messages: vec![], channel: channel, debug: vec![], has_unread: false, unread_channels: vec![]}
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }

    pub fn add_debug(&mut self, string: String) {
        self.debug.push(string);
    }

    pub fn update_unread(&mut self, view_datas: &Vec<ViewData>) {
        self.unread_channels = view_datas.iter()
            .filter_map(|vd| if vd.has_unread { Some(vd.channel.clone()) } else { None })
            .collect();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use channel::{new_from_str, Channel};

    #[test]
    fn test_update_unread_channels() {
        let mut view_data = ViewData::new(new_channel("Dev"));
        let mut other_view_data = ViewData::new(new_channel("General"));
        other_view_data.has_unread = true;
        view_data.update_unread(&vec![other_view_data]);
        assert_eq!(view_data.unread_channels.first().unwrap().name, "General");

        let mut other_view_data = ViewData::new(new_channel("General"));
        let mut other_view_data2 = ViewData::new(new_channel("Knor"));
        other_view_data.has_unread = true;
        view_data.update_unread(&vec![other_view_data, other_view_data2]);
        assert_eq!(view_data.unread_channels.len(), 1);
    }

    fn new_channel(name: &str) -> Channel {
        let json = json!({
            "id": "banana",
            "name": (name),
            "members": ["Timon"],
            "is_member": false,
            "is_general": false

        }).to_string();
        new_from_str(&json).unwrap()
    }
}
