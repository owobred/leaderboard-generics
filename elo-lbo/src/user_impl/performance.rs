use lbo::{PerformanceAttached, PerformanceAttacher};

#[derive(Clone)]
pub struct Performance;

pub struct PerformanceProcessor;

impl PerformanceProcessor {
    pub fn new() -> Self {
        Self
    }
}

impl PerformanceAttacher for PerformanceProcessor {
    type Message = super::sources::SharedMessage;

    type Performance = Performance;

    fn attach_performance(
        &self,
        _: Self::Message,
    ) -> PerformanceAttached<Self::Message, Self::Performance> {
        todo!()
    }
}
