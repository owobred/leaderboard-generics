pub struct PipelineBuilder<S, F, MetaAttach, MetrAttach, L, Msg, Metadata, Metrics>
where
    S: MessageSource<Message = Msg>,
    F: Filter<Message = Msg>,
    MetaAttach: MetadataAttacher<Message = Msg, Metadata = Metadata>,
    MetrAttach: MetricsAttacher<Message = Msg, Metadata = Metadata, Metrics = Metrics>,
    L: Leaderboard<Message = Msg, Metadata = Metadata, Metrics = Metrics>,
{
    source: Option<S>,
    filter: Option<F>,
    metadata_attacher: Option<MetaAttach>,
    metrics_attacher: Option<MetrAttach>,
    leaderboard: Option<L>,
}

impl<S, F, MetaAttach, MetrAttach, L, Msg, Metadata, Metrics>
    PipelineBuilder<S, F, MetaAttach, MetrAttach, L, Msg, Metadata, Metrics>
where
    S: MessageSource<Message = Msg>,
    F: Filter<Message = Msg>,
    MetaAttach: MetadataAttacher<Message = Msg, Metadata = Metadata>,
    MetrAttach: MetricsAttacher<Message = Msg, Metadata = Metadata, Metrics = Metrics>,
    L: Leaderboard<Message = Msg, Metadata = Metadata, Metrics = Metrics>,
{
    pub fn new() -> Self {
        Self {
            source: None,
            filter: None,
            metadata_attacher: None,
            metrics_attacher: None,
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

    pub fn metadata(mut self, metadata: MetaAttach) -> Self {
        self.metadata_attacher = Some(metadata);
        self
    }

    pub fn metrics(mut self, metrics: MetrAttach) -> Self {
        self.metrics_attacher = Some(metrics);
        self
    }

    pub fn leaderboard(mut self, leaderboard: L) -> Self {
        self.leaderboard = Some(leaderboard);
        self
    }

    pub fn build(self) -> Pipeline<S, F, MetaAttach, MetrAttach, L, Msg, Metadata, Metrics> {
        Pipeline::new(
            self.source.unwrap(),
            self.filter.unwrap(),
            self.metadata_attacher.unwrap(),
            self.metrics_attacher.unwrap(),
            self.leaderboard.unwrap(),
        )
    }
}

pub struct Pipeline<S, F, MetaAttach, MetrAttach, L, Msg, Metadata, Metrics>
where
    S: MessageSource<Message = Msg>,
    F: Filter<Message = Msg>,
    MetaAttach: MetadataAttacher<Message = Msg, Metadata = Metadata>,
    MetrAttach: MetricsAttacher<Message = Msg, Metadata = Metadata, Metrics = Metrics>,
    L: Leaderboard<Message = Msg, Metadata = Metadata, Metrics = Metrics>,
{
    source: S,
    filter: F,
    metadata_attacher: MetaAttach,
    metrics_attacher: MetrAttach,
    leaderboard: L,
}

impl<S, F, MetaAttach, MetrAttach, L, Msg, Metadata, Metrics>
    Pipeline<S, F, MetaAttach, MetrAttach, L, Msg, Metadata, Metrics>
where
    S: MessageSource<Message = Msg>,
    F: Filter<Message = Msg>,
    MetaAttach: MetadataAttacher<Message = Msg, Metadata = Metadata>,
    MetrAttach: MetricsAttacher<Message = Msg, Metadata = Metadata, Metrics = Metrics>,
    L: Leaderboard<Message = Msg, Metadata = Metadata, Metrics = Metrics>,
{
    pub fn new(
        source: S,
        filter: F,
        metadata_attacher: MetaAttach,
        metrics_attacher: MetrAttach,
        leaderboard: L,
    ) -> Self {
        Self {
            source,
            filter,
            metadata_attacher,
            metrics_attacher,
            leaderboard,
        }
    }

    pub fn run(mut self) -> Result<(), ()> {
        while let Some(msg) = self.source.next_message() {
            if !self.filter.keep_message(&msg) {
                continue;
            }

            let msg = self.metadata_attacher.attach_metadata(msg);
            let msg = self.metrics_attacher.attach_metrics(msg);

            self.leaderboard.update(&msg);
        }

        Ok(())
    }
}
pub trait MessageSource {
    type Message;

    fn next_message(&self) -> Option<Self::Message>;
}
pub trait Filter {
    type Message;

    fn keep_message(&self, message: &Self::Message) -> bool;
}
pub trait MetadataAttacher {
    type Message;
    type Metadata;

    fn attach_metadata(
        &self,
        message: Self::Message,
    ) -> MetadataAttached<Self::Message, Self::Metadata>;
}
pub trait MetricsAttacher {
    type Message;
    type Metadata;
    type Metrics;

    fn attach_metrics(
        &self,
        message: MetadataAttached<Self::Message, Self::Metadata>,
    ) -> MetricsAttached<Self::Message, Self::Metadata, Self::Metrics>;
}
pub trait Leaderboard {
    type Message;
    type Metadata;
    type Metrics;

    fn update(
        &mut self,
        performance: &MetricsAttached<Self::Message, Self::Metadata, Self::Metrics>,
    );
}

pub struct MetadataAttached<Message, Metadata> {
    pub message: Message,
    pub metadata: Metadata,
}

pub struct MetricsAttached<Message, Metadata, Metrics> {
    pub message: Message,
    pub metadata: Metadata,
    pub metrics: Metrics,
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

        return self.next.keep_message(message);
    }
}

pub struct LeaderboardSetBuilder<Msg, Meta, Metr> {
    leaderboards: Vec<Box<dyn Leaderboard<Message = Msg, Metadata = Meta, Metrics = Metr>>>,
}

impl<Msg, Meta, Metr> LeaderboardSetBuilder<Msg, Meta, Metr> {
    pub fn new() -> Self {
        Self {
            leaderboards: Vec::new(),
        }
    }

    pub fn add_leaderboard(
        mut self,
        leaderboard: impl Leaderboard<Message = Msg, Metadata = Meta, Metrics = Metr> + 'static,
    ) -> Self {
        self.leaderboards.push(Box::new(leaderboard)
            as Box<dyn Leaderboard<Message = Msg, Metadata = Meta, Metrics = Metr>>);
        self
    }

    pub fn build(self) -> LeaderboardSet<Msg, Meta, Metr> {
        LeaderboardSet::new(self.leaderboards)
    }
}

pub struct LeaderboardSet<Msg, Meta, Metr> {
    leaderboards: Vec<Box<dyn Leaderboard<Message = Msg, Metadata = Meta, Metrics = Metr>>>,
}

impl<Msg, Meta, Metr> LeaderboardSet<Msg, Meta, Metr> {
    fn new(
        leaderboards: Vec<Box<dyn Leaderboard<Message = Msg, Metadata = Meta, Metrics = Metr>>>,
    ) -> Self {
        Self { leaderboards }
    }
}

impl<Msg, Meta, Metr> Leaderboard for LeaderboardSet<Msg, Meta, Metr> {
    type Message = Msg;
    type Metadata = Meta;
    type Metrics = Metr;

    fn update(
        &mut self,
        performance: &MetricsAttached<Self::Message, Self::Metadata, Self::Metrics>,
    ) {
        for leaderboard in &mut self.leaderboards {
            leaderboard.update(performance);
        }
    }
}
