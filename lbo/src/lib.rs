pub struct PipelineBuilder<S, F, MetaAttach, L, Msg, Metadata>
where
    S: MessageSource<Message = Msg>,
    F: Filter<Message = Msg>,
    MetaAttach: MetadataAttacher<Message = Msg, Metadata = Metadata>,
    L: Leaderboard<Message = Msg, Metadata = Metadata>,
{
    source: Option<S>,
    filter: Option<F>,
    metadata_attacher: Option<MetaAttach>,
    leaderboard: Option<L>,
}

impl<S, F, MetaAttach, L, Msg, Metadata> PipelineBuilder<S, F, MetaAttach, L, Msg, Metadata>
where
    S: MessageSource<Message = Msg>,
    F: Filter<Message = Msg>,
    MetaAttach: MetadataAttacher<Message = Msg, Metadata = Metadata>,
    L: Leaderboard<Message = Msg, Metadata = Metadata>,
{
    pub fn new() -> Self {
        Self {
            source: None,
            filter: None,
            metadata_attacher: None,
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

    pub fn leaderboard(mut self, leaderboard: L) -> Self {
        self.leaderboard = Some(leaderboard);
        self
    }

    pub fn build(self) -> Pipeline<S, F, MetaAttach, L, Msg, Metadata> {
        Pipeline::new(
            self.source.unwrap(),
            self.filter.unwrap(),
            self.metadata_attacher.unwrap(),
            self.leaderboard.unwrap(),
        )
    }
}

pub struct Pipeline<S, F, MetaAttach, L, Msg, Metadata>
where
    S: MessageSource<Message = Msg>,
    F: Filter<Message = Msg>,
    MetaAttach: MetadataAttacher<Message = Msg, Metadata = Metadata>,
    L: Leaderboard<Message = Msg, Metadata = Metadata>,
{
    source: S,
    filter: F,
    metadata_attacher: MetaAttach,
    leaderboard: L,
}

impl<S, F, MetaAttach, L, Msg, Metadata> Pipeline<S, F, MetaAttach, L, Msg, Metadata>
where
    S: MessageSource<Message = Msg>,
    F: Filter<Message = Msg>,
    MetaAttach: MetadataAttacher<Message = Msg, Metadata = Metadata>,
    L: Leaderboard<Message = Msg, Metadata = Metadata>,
{
    pub fn new(source: S, filter: F, metadata_attacher: MetaAttach, leaderboard: L) -> Self {
        Self {
            source,
            filter,
            metadata_attacher,
            leaderboard,
        }
    }

    pub fn run(mut self) -> Result<(), ()> {
        while let Some(msg) = self.source.next_message() {
            if !self.filter.keep_message(&msg) {
                continue;
            }

            let msg = self.metadata_attacher.attach_metadata(msg);

            self.leaderboard.update(&msg);
        }

        Ok(())
    }
}

pub trait MessageSource {
    type Message;

    fn next_message(&mut self) -> Option<Self::Message>;
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

pub trait Leaderboard {
    type Message;
    type Metadata;

    fn update(&mut self, performance: &MetadataAttached<Self::Message, Self::Metadata>);
}

pub struct MetadataAttached<Message, Metadata> {
    pub message: Message,
    pub metadata: Metadata,
}

impl<Message, Metadata> Clone for MetadataAttached<Message, Metadata>
where
    Message: Clone,
    Metadata: Clone,
{
    fn clone(&self) -> Self {
        Self {
            message: self.message.clone(),
            metadata: self.metadata.clone(),
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

        return self.next.keep_message(message);
    }
}

pub struct LeaderboardSetBuilder<Msg, Meta> {
    leaderboards: Vec<Box<dyn Leaderboard<Message = Msg, Metadata = Meta>>>,
}

impl<Msg, Meta> LeaderboardSetBuilder<Msg, Meta> {
    pub fn new() -> Self {
        Self {
            leaderboards: Vec::new(),
        }
    }

    pub fn add_leaderboard(
        mut self,
        leaderboard: impl Leaderboard<Message = Msg, Metadata = Meta> + 'static,
    ) -> Self {
        self.leaderboards
            .push(Box::new(leaderboard) as Box<dyn Leaderboard<Message = Msg, Metadata = Meta>>);
        self
    }

    pub fn build(self) -> LeaderboardSet<Msg, Meta> {
        LeaderboardSet::new(self.leaderboards)
    }
}

pub struct LeaderboardSet<Msg, Meta> {
    leaderboards: Vec<Box<dyn Leaderboard<Message = Msg, Metadata = Meta>>>,
}

impl<Msg, Meta> LeaderboardSet<Msg, Meta> {
    fn new(leaderboards: Vec<Box<dyn Leaderboard<Message = Msg, Metadata = Meta>>>) -> Self {
        Self { leaderboards }
    }
}

impl<Msg, Meta> Leaderboard for LeaderboardSet<Msg, Meta> {
    type Message = Msg;
    type Metadata = Meta;

    fn update(&mut self, performance: &MetadataAttached<Self::Message, Self::Metadata>) {
        for leaderboard in &mut self.leaderboards {
            leaderboard.update(performance);
        }
    }
}
