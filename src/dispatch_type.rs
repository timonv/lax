#[derive(PartialEq, Debug, Clone, Hash)]
pub enum DispatchType {
    ChangeCurrentChannel,
    UserInput,
    RawIncomingMessage,
    ListChannels,
}
