pub trait Source {
    type Message;

    fn next_message(&mut self) -> impl std::future::Future<Output = Option<Self::Message>> + Send;
}
