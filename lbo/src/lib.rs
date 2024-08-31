use std::future::Future;

pub struct PipelineBuilder<S, F, PerfAttach, L, Msg, Performance>
where
    S: MessageSource<Message = Msg>,
    F: Filter<Message = Msg>,
    PerfAttach: PerformanceAttacher<Message = Msg, Performance = Performance>,
    L: Leaderboard<Message = Msg, Performance = Performance>,
{
    source: Option<S>,
    filter: Option<F>,
    performance_attacher: Option<PerfAttach>,
    leaderboard: Option<L>,
}

impl<S, F, PerfAttach, L, Msg, Performance> PipelineBuilder<S, F, PerfAttach, L, Msg, Performance>
where
    S: MessageSource<Message = Msg>,
    F: Filter<Message = Msg>,
    PerfAttach: PerformanceAttacher<Message = Msg, Performance = Performance>,
    L: Leaderboard<Message = Msg, Performance = Performance>,
{
    pub fn new() -> Self {
        Self {
            source: None,
            filter: None,
            performance_attacher: None,
            leaderboard: None,
        }
    }

    pub fn source(mut self, source: S) -> Self {
        self.source = Some(source);
        self
    }

    pub fn filter(mut self, filter: F) -> Self {
        self.filter = Some(filter);
        self
    }

    pub fn performance(mut self, performance: PerfAttach) -> Self {
        self.performance_attacher = Some(performance);
        self
    }

    pub fn leaderboard(mut self, leaderboard: L) -> Self {
        self.leaderboard = Some(leaderboard);
        self
    }

    pub fn build(self) -> Pipeline<S, F, PerfAttach, L, Msg, Performance> {
        Pipeline::new(
            self.source.unwrap(),
            self.filter.unwrap(),
            self.performance_attacher.unwrap(),
            self.leaderboard.unwrap(),
        )
    }
}

pub struct Pipeline<S, F, PerfAttach, L, Msg, Performance>
where
    S: MessageSource<Message = Msg>,
    F: Filter<Message = Msg>,
    PerfAttach: PerformanceAttacher<Message = Msg, Performance = Performance>,
    L: Leaderboard<Message = Msg, Performance = Performance>,
{
    source: S,
    filter: F,
    performance_attacher: PerfAttach,
    leaderboard: L,
}

impl<S, F, PerfAttach, L, Msg, Performance> Pipeline<S, F, PerfAttach, L, Msg, Performance>
where
    S: MessageSource<Message = Msg>,
    F: Filter<Message = Msg>,
    PerfAttach: PerformanceAttacher<Message = Msg, Performance = Performance>,
    L: Leaderboard<Message = Msg, Performance = Performance>,
{
    pub fn new(source: S, filter: F, performance_attacher: PerfAttach, leaderboard: L) -> Self {
        Self {
            source,
            filter,
            performance_attacher,
            leaderboard,
        }
    }

    // TODO: make the other functions in this loop async?
    pub async fn run(mut self) -> Result<(), ()> {
        while let Some(msg) = self.source.next_message().await {
            if !self.filter.keep_message(&msg) {
                continue;
            }

            let msg = self.performance_attacher.attach_performance(msg);

            self.leaderboard.update(&msg);
        }

        Ok(())
    }
}

pub trait MessageSource {
    type Message;

    fn next_message(&mut self) -> impl Future<Output = Option<Self::Message>> + Send + Sync;
}

pub trait Filter {
    type Message;

    fn keep_message(&self, message: &Self::Message) -> bool;
}

pub trait PerformanceAttacher {
    type Message;
    type Performance;

    fn attach_performance(
        &self,
        message: Self::Message,
    ) -> PerformanceAttached<Self::Message, Self::Performance>;
}

pub trait Leaderboard {
    type Message;
    type Performance;

    fn update(&mut self, performance: &PerformanceAttached<Self::Message, Self::Performance>);
}

pub struct PerformanceAttached<Message, Performance> {
    pub message: Message,
    pub performance: Performance,
}

impl<Message, Performance> Clone for PerformanceAttached<Message, Performance>
where
    Message: Clone,
    Performance: Clone,
{
    fn clone(&self) -> Self {
        Self {
            message: self.message.clone(),
            performance: self.performance.clone(),
        }
    }
}

pub struct FilterChainBuilder<M> {
    filters: Vec<Box<dyn Filter<Message = M>>>,
}

impl<M> FilterChainBuilder<M> {
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
        }
    }

    pub fn add_filter(mut self, filter: impl Filter<Message = M> + 'static) -> Self {
        self.filters
            .push(Box::new(filter) as Box<dyn Filter<Message = M>>);
        self
    }

    pub fn build(self) -> FilterChain<M> {
        FilterChain::new(self.filters)
    }
}

pub struct FilterChain<M> {
    filters: Vec<Box<dyn Filter<Message = M>>>,
}

impl<M> Filter for FilterChain<M> {
    type Message = M;

    fn keep_message(&self, message: &Self::Message) -> bool {
        for filter in &self.filters {
            if !filter.keep_message(message) {
                return false;
            }
        }

        true
    }
}

impl<M> FilterChain<M> {
    fn new(filters: Vec<Box<dyn Filter<Message = M>>>) -> Self {
        Self { filters }
    }
}

pub struct NullFilter<M> {
    _phantom: std::marker::PhantomData<M>,
}

impl<M> Filter for NullFilter<M> {
    type Message = M;

    fn keep_message(&self, _: &Self::Message) -> bool {
        true
    }
}

impl<M> NullFilter<M> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

pub struct StaticFilterSet<H, N, M>
where
    H: Filter<Message = M>,
    N: Filter<Message = M>,
{
    head: H,
    next: N,
}

impl<H, M> StaticFilterSet<H, NullFilter<M>, M>
where
    H: Filter<Message = M>,
{
    pub fn new(head: H) -> Self {
        Self {
            head,
            next: NullFilter::new(),
        }
    }
}

impl<H, M> StaticFilterSet<H, NullFilter<M>, M>
where
    H: Filter<Message = M>,
{
    pub fn append<N: Filter<Message = M>>(
        self,
        next: N,
    ) -> StaticFilterSet<H, StaticFilterSet<N, NullFilter<M>, M>, M> {
        StaticFilterSet {
            head: self.head,
            next: StaticFilterSet {
                head: next,
                next: NullFilter::new(),
            },
        }
    }
}

impl<H, N, M> Filter for StaticFilterSet<H, N, M>
where
    H: Filter<Message = M>,
    N: Filter<Message = M>,
{
    type Message = M;

    fn keep_message(&self, message: &Self::Message) -> bool {
        if !self.head.keep_message(message) {
            return false;
        }

        self.next.keep_message(message)
    }
}

pub struct LeaderboardSetBuilder<Msg, Perf> {
    leaderboards: Vec<Box<dyn Leaderboard<Message = Msg, Performance = Perf>>>,
}

impl<Msg, Perf> LeaderboardSetBuilder<Msg, Perf> {
    pub fn new() -> Self {
        Self {
            leaderboards: Vec::new(),
        }
    }

    pub fn add_leaderboard(
        mut self,
        leaderboard: impl Leaderboard<Message = Msg, Performance = Perf> + 'static,
    ) -> Self {
        self.leaderboards
            .push(Box::new(leaderboard) as Box<dyn Leaderboard<Message = Msg, Performance = Perf>>);
        self
    }

    pub fn build(self) -> LeaderboardSet<Msg, Perf> {
        LeaderboardSet::new(self.leaderboards)
    }
}

pub struct LeaderboardSet<Msg, Perf> {
    leaderboards: Vec<Box<dyn Leaderboard<Message = Msg, Performance = Perf>>>,
}

impl<Msg, Perf> LeaderboardSet<Msg, Perf> {
    fn new(leaderboards: Vec<Box<dyn Leaderboard<Message = Msg, Performance = Perf>>>) -> Self {
        Self { leaderboards }
    }
}

impl<Msg, Perf> Leaderboard for LeaderboardSet<Msg, Perf> {
    type Message = Msg;
    type Performance = Perf;

    fn update(&mut self, performance: &PerformanceAttached<Self::Message, Self::Performance>) {
        for leaderboard in &mut self.leaderboards {
            leaderboard.update(performance);
        }
    }
}
