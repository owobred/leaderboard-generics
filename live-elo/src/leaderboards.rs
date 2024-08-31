pub struct NullLeaderboard<M, P> {
    _phantom: std::marker::PhantomData<(M, P)>,
}

impl<M, P> NullLeaderboard<M, P> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<M, P> lbo::Leaderboard for NullLeaderboard<M, P> {
    type Message = M;
    type Performance = P;

    fn update(
        &mut self,
        _performance: &lbo::PerformanceAttached<Self::Message, Self::Performance>,
    ) {
    }
}
