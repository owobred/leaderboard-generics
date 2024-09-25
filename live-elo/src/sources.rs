use lbo::{message::AuthoredMesasge, sources::Source};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::{trace, warn};
use websocket_shared::{AuthorId, TwitchId};

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
    type Closed = ();

    async fn next_message(&mut self) -> Option<Self::Message> {
        self.mpsc_recv
            .recv()
            .await
            .map(|message| Message::Twitch(message))
    }

    async fn close(self) -> Self::Closed {
        drop(self.mpsc_recv);

        self.task_join.abort();

        match self.task_join.await {
            Ok(_) => (),
            Err(error) => warn!(?error, "error whilst closing aborted twitch message source"),
        }
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
                            // FIXME: this should be the users id, I just changed it to make it wayy more clear in the web thing
                            //        beacuse I'm too lazy to properly send this data over an api or something sane
                            author_id: message.sender.login,
                            // author_id: message.sender.id,
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

impl<S, M, C> CancellableSource<S, M, C> where S: Source<Message = M, Closed = C> {
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
