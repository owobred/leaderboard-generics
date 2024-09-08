use lbo::{message::AuthoredMesasge, sources::Source};

#[derive(Debug)]
pub enum Message {
    Twitch { message: String, author_id: String },
}

pub struct DummyTwitchSource {
    val: std::sync::atomic::AtomicUsize,
}

impl DummyTwitchSource {
    pub fn new() -> Self {
        Self {
            val: std::sync::atomic::AtomicUsize::new(0),
        }
    }
}

impl Source for DummyTwitchSource {
    type Message = Message;

    async fn next_message(&self) -> Option<Self::Message> {
        let value = self.val.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if value < 10 {
            Some(Message::Twitch {
                message: value.to_string(),
                author_id: "123123123".to_string(),
            })
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub enum AuthorId {
    Twitch(String),
}

impl AuthoredMesasge for Message {
    type Id = AuthorId;

    fn author_id(&self) -> Self::Id {
        match self {
            Message::Twitch { author_id, .. } => AuthorId::Twitch(author_id.clone()),
        }
    }
}
