use lbo::{Leaderboard, PerformanceAttached};

pub struct AsyncLeaderboardSetBuilder<Message, Performance>
where
    Message: Clone + Send + Sync + 'static,
    Performance: Clone + Send + Sync + 'static,
{
    performance_send: tokio::sync::broadcast::Sender<PerformanceAttached<Message, Performance>>,
    performance_recv: tokio::sync::broadcast::Receiver<PerformanceAttached<Message, Performance>>,
    joinset: tokio::task::JoinSet<()>,
}

impl<Message, Performance> AsyncLeaderboardSetBuilder<Message, Performance>
where
    Message: Clone + Send + Sync + 'static,
    Performance: Clone + Send + Sync + 'static,
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
        leaderboard: impl Leaderboard<Message = Message, Performance = Performance>
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
        AsyncLeaderboardSet<Message, Performance>,
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

async fn leaderboard_task<S, Message, Performance>(
    mut leaderboard: S,
    mut broadcast: tokio::sync::broadcast::Receiver<PerformanceAttached<Message, Performance>>,
) where
    S: Leaderboard<Message = Message, Performance = Performance>,
    Message: Clone + Send + Sync + 'static,
    Performance: Clone + Send + Sync + 'static,
{
    while let Ok(msg) = broadcast.recv().await {
        leaderboard.update(&msg);
    }
}

pub struct AsyncLeaderboardSet<Message, Performance>
where
    Message: Clone + Send + Sync + 'static,
    Performance: Clone + Send + Sync + 'static,
{
    performance_send: tokio::sync::broadcast::Sender<PerformanceAttached<Message, Performance>>,
}

impl<Message, Performance> Leaderboard for AsyncLeaderboardSet<Message, Performance>
where
    Message: Clone + Send + Sync + 'static,
    Performance: Clone + Send + Sync + 'static,
{
    type Message = Message;
    type Performance = Performance;

    fn update(&mut self, performance: &PerformanceAttached<Self::Message, Self::Performance>) {
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
