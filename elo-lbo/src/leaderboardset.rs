use lbo::{Leaderboard, MetadataAttached};

pub struct AsyncLeaderboardSetBuilder<Message, Metadata>
where
    Message: Clone + Send + Sync + 'static,
    Metadata: Clone + Send + Sync + 'static,
{
    performance_send: tokio::sync::broadcast::Sender<MetadataAttached<Message, Metadata>>,
    performance_recv: tokio::sync::broadcast::Receiver<MetadataAttached<Message, Metadata>>,
    joinset: tokio::task::JoinSet<()>,
}

impl<Message, Metadata> AsyncLeaderboardSetBuilder<Message, Metadata>
where
    Message: Clone + Send + Sync + 'static,
    Metadata: Clone + Send + Sync + 'static,
{
    pub fn new() -> Self {
        let (performance_send, performance_recv) = tokio::sync::broadcast::channel(1_000_000);

        Self {
            performance_send,
            performance_recv,
            joinset: tokio::task::JoinSet::new(),
        }
    }

    pub fn spawn_leaderboard(
        mut self,
        leaderboard: impl Leaderboard<Message = Message, Metadata = Metadata>
            + Send
            + Sync
            + 'static,
    ) -> Self {
        self.joinset.spawn(leaderboard_task(
            leaderboard,
            self.performance_recv.resubscribe(),
        ));
        self
    }

    pub fn build(
        self,
    ) -> (
        AsyncLeaderboardSet<Message, Metadata>,
        AsyncLeaderboardsHandle,
    ) {
        (
            AsyncLeaderboardSet {
                performance_send: self.performance_send,
            },
            AsyncLeaderboardsHandle {
                joinset: self.joinset,
            },
        )
    }
}

async fn leaderboard_task<S, Message, Metadata>(
    mut leaderboard: S,
    mut broadcast: tokio::sync::broadcast::Receiver<MetadataAttached<Message, Metadata>>,
) where
    S: Leaderboard<Message = Message, Metadata = Metadata>,
    Message: Clone + Send + Sync + 'static,
    Metadata: Clone + Send + Sync + 'static,
{
    while let Ok(msg) = broadcast.recv().await {
        leaderboard.update(&msg);
    }
}

pub struct AsyncLeaderboardSet<Message, Metadata>
where
    Message: Clone + Send + Sync + 'static,
    Metadata: Clone + Send + Sync + 'static,
{
    performance_send: tokio::sync::broadcast::Sender<MetadataAttached<Message, Metadata>>,
}

impl<Message, Metadata> Leaderboard for AsyncLeaderboardSet<Message, Metadata>
where
    Message: Clone + Send + Sync + 'static,
    Metadata: Clone + Send + Sync + 'static,
{
    type Message = Message;
    type Metadata = Metadata;

    fn update(
        &mut self,
        performance: &MetadataAttached<Self::Message, Self::Metadata>,
    ) {
        self.performance_send
            .send(performance.to_owned())
            .ok()
            .unwrap();
    }
}

pub struct AsyncLeaderboardsHandle {
    joinset: tokio::task::JoinSet<()>,
}

impl AsyncLeaderboardsHandle {
    pub async fn join(mut self) {
        while let Some(result) = self.joinset.join_next().await {
            result.unwrap()
        }
    }
}
