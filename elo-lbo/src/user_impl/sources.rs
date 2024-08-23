use lbo::MessageSource;

pub struct TwitchMessageSource {}

impl MessageSource for TwitchMessageSource {
    type Message = SharedMessage;

    fn next_message(&self) -> Option<Self::Message> {
        todo!()
    }
}

impl TwitchMessageSource {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct DiscordMessageSource {}

impl MessageSource for DiscordMessageSource {
    type Message = SharedMessage;

    fn next_message(&self) -> Option<Self::Message> {
        todo!()
    }
}

impl DiscordMessageSource {
    pub fn new() -> Self {
        Self {}
    }
}

pub enum SharedMessage {
    Twitch(()),
    Discord(()),
}
