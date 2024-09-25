pub mod twitch;

use lbo::{message::AuthoredMesasge, sources::Source};
use tokio_util::sync::CancellationToken;
use twitch::TwitchMessage;
use websocket_shared::{AuthorId, TwitchId};

#[derive(Debug)]
pub enum Message {
    Twitch(TwitchMessage),
}

impl AuthoredMesasge for Message {
    type Id = AuthorId;

    fn author_id(&self) -> Self::Id {
        match self {
            Message::Twitch(message) => AuthorId::Twitch(TwitchId::new(message.author_id.clone())),
        }
    }
}

pub struct CancellableSource<S, M, C>
where
    S: Source<Message = M, Closed = C>,
{
    source: S,
    cancellation_token: CancellationToken,
}

impl<S, M, C> CancellableSource<S, M, C>
where
    S: Source<Message = M, Closed = C>,
{
    pub fn new(source: S, cancellation_token: CancellationToken) -> Self {
        Self {
            source,
            cancellation_token,
        }
    }
}

impl<S, M, C> Source for CancellableSource<S, M, C>
where
    S: Source<Message = M, Closed = C> + Send,
{
    type Message = M;
    type Closed = C;

    async fn next_message(&mut self) -> Option<Self::Message> {
        tokio::select! {
            message = self.source.next_message() => message,
            _ = self.cancellation_token.cancelled() => None,
        }
    }

    async fn close(self) -> Self::Closed {
        self.source.close().await
    }
}
