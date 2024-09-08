pub mod exporter;
pub mod filter;
pub mod message;
pub mod performances;
pub mod scoring;
pub mod sources;

pub struct PipelineBuilder<Source, Filter, Performances, Message>
where
    Source: sources::Source<Message = Message>,
    Filter: filter::Filter<Message = Message>,
    Performances: performances::PerformanceProcessor<Message = Message>,
{
    source: Option<Source>,
    filter: Option<Filter>,
    performances: Option<Performances>,
}

impl<Source, Filter, Performances, Message> PipelineBuilder<Source, Filter, Performances, Message>
where
    Source: sources::Source<Message = Message>,
    Filter: filter::Filter<Message = Message>,
    Performances: performances::PerformanceProcessor<Message = Message>,
{
    pub fn new() -> Self {
        Self {
            source: None,
            filter: None,
            performances: None,
        }
    }

    pub fn source(mut self, source: Source) -> Self {
        self.source = Some(source);
        self
    }

    pub fn filter(mut self, filter: Filter) -> Self {
        self.filter = Some(filter);
        self
    }

    pub fn performances(mut self, performances: Performances) -> Self {
        self.performances = Some(performances);
        self
    }

    pub fn build(self) -> Pipeline<Source, Filter, Performances, Message> {
        Pipeline::new(
            self.source.unwrap(),
            self.filter.unwrap(),
            self.performances.unwrap(),
        )
    }
}

impl<Source, Filter, Performances, Message> Default
    for PipelineBuilder<Source, Filter, Performances, Message>
where
    Source: sources::Source<Message = Message>,
    Filter: filter::Filter<Message = Message>,
    Performances: performances::PerformanceProcessor<Message = Message>,
{
    fn default() -> Self {
        Self::new()
    }
}

pub struct Pipeline<Source, Filter, Performances, Message>
where
    Source: sources::Source<Message = Message>,
    Filter: filter::Filter<Message = Message>,
    Performances: performances::PerformanceProcessor<Message = Message>,
{
    source: Source,
    filter: Filter,
    performances: Performances,
}

impl<Source, Filter, Performances, Message> Pipeline<Source, Filter, Performances, Message>
where
    Source: sources::Source<Message = Message>,
    Filter: filter::Filter<Message = Message>,
    Performances: performances::PerformanceProcessor<Message = Message>,
{
    pub fn builder() -> PipelineBuilder<Source, Filter, Performances, Message> {
        PipelineBuilder::new()
    }

    pub fn new(source: Source, filter: Filter, performances: Performances) -> Self {
        Self {
            source,
            filter,
            performances,
        }
    }

    pub async fn run(mut self) -> Result<Self, ()> {
        while let Some(message) = self.source.next_message().await {
            if !self.filter.keep(&message) {
                continue;
            }

            self.performances.process_message(message).await;
        }

        Ok(self)
    }
}
