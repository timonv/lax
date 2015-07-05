#[derive(PartialEq, Debug, Clone)]
pub enum DispatchType {
    ChangeCurrentChannel,
    UserInput,
    RawIncomingMessage,
    ListChannels,
}
