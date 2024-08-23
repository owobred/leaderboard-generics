use lbo::MessageSource;

pub struct AsyncSourcesBuilder<M> {
    mpsc_send: tokio::sync::mpsc::Sender<M>,
    mpsc_recv: tokio::sync::mpsc::Receiver<M>,
    joinset: tokio::task::JoinSet<()>,
}

impl<M> AsyncSourcesBuilder<M>
where
    M: Send + Sync + 'static,
{
    pub fn new() -> Self {
        let (mpsc_send, mpsc_recv) = tokio::sync::mpsc::channel(1_000_000);
        Self {
            mpsc_send,
            mpsc_recv,
            joinset: tokio::task::JoinSet::new(),
        }
    }

    pub fn spawn_source(
        mut self,
        source: impl MessageSource<Message = M> + Send + Sync + 'static,
    ) -> Self {
        self.joinset
            .spawn(source_task(source, self.mpsc_send.clone()));
        self
    }

    pub fn build(self) -> (AsyncSourcesRecv<M>, AsyncSourcesHandle) {
        (
            AsyncSourcesRecv {
                mpsc_recv: self.mpsc_recv,
            },
            AsyncSourcesHandle {
                joinset: self.joinset,
            },
        )
    }
}

async fn source_task<S, M>(mut source: S, mpsc: tokio::sync::mpsc::Sender<M>)
where
    S: MessageSource<Message = M>,
{
    while let Some(msg) = source.next_message() {
        mpsc.send(msg).await.unwrap();
    }
}

pub struct AsyncSourcesRecv<M> {
    mpsc_recv: tokio::sync::mpsc::Receiver<M>,
}

impl<M> MessageSource for AsyncSourcesRecv<M> {
    type Message = M;

    fn next_message(&mut self) -> Option<Self::Message> {
        self.mpsc_recv.blocking_recv()
    }
}

pub struct AsyncSourcesHandle {
    joinset: tokio::task::JoinSet<()>,
}

impl AsyncSourcesHandle {
    pub async fn join(mut self) {
        while let Some(result) = self.joinset.join_next().await {
            result.unwrap()
        }
    }
}
