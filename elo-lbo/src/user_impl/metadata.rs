use lbo::{MetadataAttached, MetadataAttacher};

pub struct Metadata;

pub struct MetadataProcessor;

impl MetadataProcessor {
    pub fn new() -> Self {
        Self
    }
}

impl MetadataAttacher for MetadataProcessor {
    type Message = super::sources::SharedMessage;

    type Metadata = Metadata;

    fn attach_metadata(
        &self,
        message: Self::Message,
    ) -> MetadataAttached<Self::Message, Self::Metadata> {
        todo!()
    }
}