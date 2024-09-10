use lbo::scoring::ScoringSystem;

pub struct DummyScoring {}

impl DummyScoring {
    pub fn new() -> Self {
        Self {}
    }
}

impl ScoringSystem for DummyScoring {
    type Message = super::sources::Message;
    type Performance = super::exporter::websocket::PerformancePoints;

    fn score_message(&self, message: Self::Message) -> Self::Performance {
        match message {
            crate::sources::Message::Twitch { message, .. } => {
                super::exporter::websocket::PerformancePoints::new(message.parse().unwrap())
            }
        }
    }
}
