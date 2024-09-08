pub trait Exporter {
    type Performance;
    type AuthorId;

    fn export(&mut self, author_id: Self::AuthorId, performance: Self::Performance) -> impl std::future::Future<Output = ()>;
}
