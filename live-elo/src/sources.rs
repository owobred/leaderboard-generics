use lbo::{message::AuthoredMesasge, sources::Source};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::trace;

#[derive(Debug)]
pub struct TwitchMessage {
    message: String,
    author_id: String,
}

#[derive(Debug)]
pub enum Message {
    Twitch(TwitchMessage),
}

pub struct TwitchMessageSourceHandle {
    mpsc_recv: mpsc::Receiver<TwitchMessage>,
    task_join: tokio::task::JoinHandle<()>,
}

impl TwitchMessageSourceHandle {
    pub fn spawn() -> Self {
        let (mpsc_send, mpsc_recv) = mpsc::channel(1000);
        let task_join = tokio::task::spawn(twitch_source_inner(mpsc_send));

        Self {
            mpsc_recv,
            task_join,
        }
    }

    // TODO: There should probably be some kind of way to clean this up in the pipeline...
    //       might be worth providing something like `Pipeline::cleanup(self)` which then calls async cleanup
    //       on sub components
    pub async fn cancel(self) {
        drop(self.mpsc_recv);
        self.task_join.abort();
    }
}

impl Source for TwitchMessageSourceHandle {
    type Message = Message;

    async fn next_message(&mut self) -> Option<Self::Message> {
        self.mpsc_recv
            .recv()
            .await
            .map(|message| Message::Twitch(message))
    }
}

async fn twitch_source_inner(mpsc_send: mpsc::Sender<TwitchMessage>) {
    let config = twitch_irc::ClientConfig::default();
    let (mut incoming_message, client) =
        twitch_irc::TwitchIRCClient::<twitch_irc::SecureTCPTransport, _>::new(config);

    let jh = tokio::task::spawn(async move {
        while let Some(message) = incoming_message.recv().await {
            match message {
                twitch_irc::message::ServerMessage::Privmsg(message) => {
                    trace!(?message, "got irc message");
                    mpsc_send
                        .send(TwitchMessage {
                            message: message.message_text,
                            author_id: message.sender.id,
                        })
                        .await
                        .unwrap();
                }
                _ => (),
            }
        }
    });

    // FIXME: this should be behind some kinda of config I beg of you
    client.join("ironmouse".to_string()).unwrap();

    jh.await.unwrap()
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
            Message::Twitch(message) => AuthorId::Twitch(TwitchId::new(message.author_id.clone())),
        }
    }
}

impl From<TwitchId> for AuthorId {
    fn from(value: TwitchId) -> Self {
        Self::Twitch(value)
    }
}
