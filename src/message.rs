use serialize::json::{self, Json, DecodeResult};
use serialize::{Decodable, Decoder};

use user::User;
use channel::Channel;

// All EventTypes typed out into enum
// Not used currently
enum EventType {
    Hello,
    Message,
    UserTyping,
    ChannelMarked,
    ChannelCreated,
    ChannelJoined,
    ChannelLeft,
    ChannelDeleted,
    ChannelRename,
    ChannelArchive,
    ChannelUnarchive,
    ChannelHistoryChanged,
    ImCreated,
    ImOpen,
    ImClose,
    ImMarked,
    ImHistoryChanged,
    GroupJoined,
    GroupLeft,
    GroupOpen,
    GroupClose,
    GroupArchive,
    GroupUnarchive,
    GroupRename,
    GroupMarked,
    GroupHistoryChanged,
    FileCreated,
    FileShared,
    FileUnshared,
    FilePublic,
    FilePrivate,
    FileChange,
    FileDeleted,
    FileCommentAdded,
    FileCommentEdited,
    FileCommentDeleted,
    PinAdded,
    PinRemoved,
    PresenceChange,
    ManualPresenceChange,
    PrefChange,
    UserChange,
    TeamJoin,
    StarAdded,
    StarRemoved,
    EmojiChanged,
    CommandsChanged,
    TeamPlanChange,
    TeamPrefChange,
    TeamRename,
    TeamDomainChange,
    EmailDomainChange,
    BotAdded,
    BotChanged,
    AccountsChanged,
    TeamMigrationStarted
}

pub struct Message {
    pub ts: Option<String>,
    pub text: Option<String>,
    pub user: Option<User>,
    pub channel: Option<Channel>,
    pub event_type: Option<String>,// TODO Enum
    pub user_id: Option<String>,
    pub channel_id: Option<String>,
    // payload: String

}

pub fn new_message_from_str(payload: &str) -> DecodeResult<Message> {
    json::decode::<Message>(payload)
}

impl Decodable for Message {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Message, D::Error> {
        decoder.read_struct("root", 0, |decoder| {
            Ok(Message {
                event_type: try!(decoder.read_struct_field("type", 0, |decoder| Decodable::decode(decoder))),
                user_id: try!(decoder.read_struct_field("user", 0, |decoder| Decodable::decode(decoder))),
                text: try!(decoder.read_struct_field("text", 0, |decoder| Decodable::decode(decoder))),
                ts: try!(decoder.read_struct_field("ts", 0, |decoder| Decodable::decode(decoder))),
                channel_id: try!(decoder.read_struct_field("channel", 0, |decoder| Decodable::decode(decoder))),
                channel: None,
                user: None
            })
        })
    }
}

#[cfg(test)]
mod test {
    use super::new_message_from_str;

    #[test]
    fn test_decode_from_json() {
        let json = "{
            \"type\": \"message\",
            \"user\": \"Timon\",
            \"text\": \"Bananas!\",
            \"ts\": \"today\",
            \"channel\": \"banter\"
        }";
        let message = new_message_from_str(json).unwrap();
        assert_eq!(message.event_type.unwrap(), "message");
        assert_eq!(message.user_id.unwrap(), "Timon");
        assert_eq!(message.channel_id.unwrap(), "banter");
        assert_eq!(message.text.unwrap(), "Bananas!");
        assert_eq!(message.ts.unwrap(), "today");
    }
}

