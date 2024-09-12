pub mod websocket;
pub mod elo_calculator;

use lbo::exporter::Exporter;
use websocket::PerformancePoints;
use websocket_shared::AuthorId;

pub struct DummyExporter {}

impl DummyExporter {
    pub fn new() -> Self {
        Self {}
    }
}

impl Exporter for DummyExporter {
    type Performance = PerformancePoints;
    type AuthorId = AuthorId;

    async fn export(&mut self, author_id: Self::AuthorId, performance: Self::Performance) {
        println!("got performance for {author_id:?}: {performance:?}");
    }
}

pub struct MultiExporter<Head, Tail, Performance, AuthorId>
where
    Head: Exporter<Performance = Performance, AuthorId = AuthorId>,
    Tail: Exporter<Performance = Performance, AuthorId = AuthorId>,
    Performance: Clone,
    AuthorId: Clone,
{
    head: Head,
    tail: Tail,
}

impl<Head, Tail, Performance, AuthorId> MultiExporter<Head, Tail, Performance, AuthorId>
where
    Head: Exporter<Performance = Performance, AuthorId = AuthorId>,
    Tail: Exporter<Performance = Performance, AuthorId = AuthorId>,
    Performance: Clone,
    AuthorId: Clone,
{
    pub fn pair(head: Head, tail: Tail) -> Self {
        Self { head, tail }
    }

    pub fn append<T>(
        self,
        value: T,
    ) -> MultiExporter<T, MultiExporter<Head, Tail, Performance, AuthorId>, Performance, AuthorId>
    where
        T: Exporter<Performance = Performance, AuthorId = AuthorId>,
    {
        MultiExporter::pair(value, self)
    }
}

impl<Head, Tail, Performance, AuthorId> Exporter
    for MultiExporter<Head, Tail, Performance, AuthorId>
where
    Head: Exporter<Performance = Performance, AuthorId = AuthorId>,
    Tail: Exporter<Performance = Performance, AuthorId = AuthorId>,
    Performance: Clone,
    AuthorId: Clone,
{
    type Performance = Performance;
    type AuthorId = AuthorId;

    async fn export(&mut self, author_id: Self::AuthorId, performance: Self::Performance) {
        self.head
            .export(author_id.clone(), performance.clone())
            .await;
        self.tail.export(author_id, performance).await;
    }
}
