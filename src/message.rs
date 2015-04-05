use serialize::json::{self, Json, DecodeResult};
use serialize::{Decodable, Decoder};

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
    pub event_type: Option<String>,// TODO Enum
    pub user: Option<String>,
    pub text: Option<String>,
    pub ts: Option<String>,
    pub channel: Option<String>
}

pub fn new_message_from_str(payload: &str) -> DecodeResult<Message> {
    json::decode::<Message>(payload)
}

impl Decodable for Message {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Message, D::Error> {
        decoder.read_struct("root", 0, |decoder| {
            Ok(Message {
                event_type: try!(decoder.read_struct_field("type", 0, |decoder| Decodable::decode(decoder))),
                user: try!(decoder.read_struct_field("user", 0, |decoder| Decodable::decode(decoder))),
                text: try!(decoder.read_struct_field("text", 0, |decoder| Decodable::decode(decoder))),
                ts: try!(decoder.read_struct_field("ts", 0, |decoder| Decodable::decode(decoder))),
                channel: try!(decoder.read_struct_field("channel", 0, |decoder| Decodable::decode(decoder))),
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
        assert_eq!(message.user.unwrap(), "Timon");
        assert_eq!(message.text.unwrap(), "Bananas!");
        assert_eq!(message.ts.unwrap(), "today");
        assert_eq!(message.channel.unwrap(), "banter");
    }
}

