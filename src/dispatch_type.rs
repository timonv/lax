#[derive(PartialEq, Debug, Clone)]
pub enum DispatchType {
    ChangeCurrentChannel,
    OutgoingMessage,
    RawIncomingMessage,
    ListChannels,
    UserInput,
}
