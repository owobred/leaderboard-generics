pub trait Filter {
    type Message;

    fn keep(&self, message: &Self::Message) -> bool;
}
