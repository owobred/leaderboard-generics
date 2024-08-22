use crate::MetricsAttacher;

pub struct Metrics;

pub struct MetricsProcessor;

impl MetricsProcessor {
    pub fn new() -> Self {
        Self {}
    }
}

impl MetricsAttacher for MetricsProcessor {
    type Message = super::sources::SharedMessage;
    type Metadata = super::metadata::Metadata;
    type Metrics = Metrics;

    fn attach_metrics(
        &self,
        message: crate::MetadataAttached<Self::Message, Self::Metadata>,
    ) -> crate::MetricsAttached<Self::Message, Self::Metadata, Self::Metrics> {
        todo!()
    }
}
