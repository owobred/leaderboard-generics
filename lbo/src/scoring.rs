pub trait ScoringSystem {
    type Message;
    type Performance;

    fn score_message(&self, message: Self::Message) -> Self::Performance;
}
