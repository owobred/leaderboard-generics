use crate::{message::AuthoredMesasge, scoring::ScoringSystem};

pub trait PerformanceProcessor {
    type Message;

    fn process_message(&mut self, message: Self::Message) -> impl std::future::Future<Output = ()>;
}

pub struct StandardLeaderboard<Scoring, Exporter, Message, Id, Performance>
where
    Message: AuthoredMesasge<Id = Id>,
    Scoring: ScoringSystem<Message = Message, Performance = Performance>,
    Exporter: crate::exporter::Exporter<AuthorId = Id, Performance = Performance>,
{
    scoring_system: Scoring,
    exporter: Exporter,
}

impl<Scoring, Exporter, Message, Id, Performance>
    StandardLeaderboard<Scoring, Exporter, Message, Id, Performance>
where
    Message: AuthoredMesasge<Id = Id>,
    Scoring: ScoringSystem<Message = Message, Performance = Performance>,
    Exporter: crate::exporter::Exporter<AuthorId = Id, Performance = Performance>,
{
    pub fn new(scoring_system: Scoring, exporter: Exporter) -> Self {
        Self {
            scoring_system,
            exporter,
        }
    }
}

impl<Scoring, Exporter, Message, Id, Performance> PerformanceProcessor
    for StandardLeaderboard<Scoring, Exporter, Message, Id, Performance>
where
    Message: AuthoredMesasge<Id = Id>,
    Scoring: ScoringSystem<Message = Message, Performance = Performance>,
    Exporter: crate::exporter::Exporter<AuthorId = Id, Performance = Performance>,
{
    type Message = Message;

    async fn process_message(&mut self, message: Self::Message) {
        let message_author_id = message.author_id();
        let score = self.scoring_system.score_message(message);
        self.exporter.export(message_author_id, score).await;
    }
}
