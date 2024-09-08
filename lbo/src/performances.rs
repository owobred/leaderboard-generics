use crate::{message::AuthoredMesasge, scoring::ScoringSystem};

pub trait PerformanceProcessor {
    type Message;

    fn process_message(&mut self, message: Self::Message) -> impl std::future::Future<Output = ()>;
}

pub struct StandardLeaderboard<Scoring, Exporter, Message, Id>
where
    Message: AuthoredMesasge<Id = Id>,
    Scoring: ScoringSystem<Message = Message>,
    Exporter: crate::exporter::Exporter<AuthorId = Id, Performance = f32>,
{
    scoring_system: Scoring,
    exporter: Exporter,
}

impl<Scoring, Exporter, Message, Id> StandardLeaderboard<Scoring, Exporter, Message, Id>
where
    Message: AuthoredMesasge<Id = Id>,
    Scoring: ScoringSystem<Message = Message>,
    Exporter: crate::exporter::Exporter<AuthorId = Id, Performance = f32>,
{
    pub fn new(scoring_system: Scoring, exporter: Exporter) -> Self {
        Self {
            scoring_system,
            exporter,
        }
    }
}

impl<Scoring, Exporter, Message, Id> PerformanceProcessor
    for StandardLeaderboard<Scoring, Exporter, Message, Id>
where
    Message: AuthoredMesasge<Id = Id>,
    Scoring: ScoringSystem<Message = Message>,
    Exporter: crate::exporter::Exporter<AuthorId = Id, Performance = f32>,
{
    type Message = Message;

    async fn process_message(&mut self, message: Self::Message) {
        let message_author_id = message.author_id();
        let score = self.scoring_system.score_message(message);
        self.exporter.export(message_author_id, score).await;
    }
}
