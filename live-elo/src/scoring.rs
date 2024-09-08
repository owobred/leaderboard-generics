use lbo::scoring::ScoringSystem;

pub struct DummyScoring {}

impl DummyScoring {
    pub fn new() -> Self {
        Self {}
    }
}

impl ScoringSystem for DummyScoring {
    type Message = super::sources::Message;

    fn score_message(&self, message: Self::Message) -> f32 {
        match message {
            crate::sources::Message::Twitch { message, .. } => message.parse().unwrap(),
        }
    }
}
