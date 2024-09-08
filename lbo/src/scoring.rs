pub trait ScoringSystem {
    type Message;

    fn score_message(&self, message: Self::Message) -> f32;
}
