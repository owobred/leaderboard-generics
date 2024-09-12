use lbo::scoring::ScoringSystem;

pub struct MessageCountScoring {}

impl MessageCountScoring {
    pub fn new() -> Self {
        Self {}
    }
}

impl ScoringSystem for MessageCountScoring {
    type Message = super::sources::Message;
    type Performance = super::exporter::websocket::PerformancePoints;

    fn score_message(&self, message: Self::Message) -> Self::Performance {
        match message {
            crate::sources::Message::Twitch(_) => {
                super::exporter::websocket::PerformancePoints::new(1.0)
            }
        }
    }
}
