use lbo::{message::AuthoredMesasge, sources::Source};
use serde::{Deserialize, Serialize};

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
        // if value < 10 {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            Some(Message::Twitch {
                message: value.to_string(),
                author_id: "123123123".to_string(),
            })
        // } else {
            // None
        // }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TwitchId(String);

impl TwitchId {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    pub fn get(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(tag = "platform", content = "id", rename_all = "snake_case")]
pub enum AuthorId {
    Twitch(TwitchId),
}

impl AuthoredMesasge for Message {
    type Id = AuthorId;

    fn author_id(&self) -> Self::Id {
        match self {
            Message::Twitch { author_id, .. } => AuthorId::Twitch(TwitchId::new(author_id.clone())),
        }
    }
}

impl From<TwitchId> for AuthorId {
    fn from(value: TwitchId) -> Self {
        Self::Twitch(value)
    }
}
