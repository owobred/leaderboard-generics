use lbo::exporter::Exporter;

pub struct DummyExporter {}

impl DummyExporter {
    pub fn new() -> Self {
        Self {}
    }
}

impl Exporter for DummyExporter {
    type Performance = f32;
    type AuthorId = super::sources::AuthorId;

    async fn export(&mut self, author_id: Self::AuthorId, performance: Self::Performance) {
        println!("got performance for {author_id:?}: {performance:?}");
    }
}
