pub mod twitch;

#[derive(Debug, Clone)]
pub enum Message {
    Twitch(twitch::Message),
}