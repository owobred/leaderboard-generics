use lbo::{Leaderboard, MetadataAttached};

#[derive(Clone)]
pub struct BitsOnly {
    state: (),
}

impl BitsOnly {
    pub fn new() -> Self {
        Self { state: () }
    }
}

impl Leaderboard for BitsOnly {
    type Message = super::sources::SharedMessage;
    type Metadata = super::metadata::Metadata;

    fn update(
        &mut self,
        _: &MetadataAttached<Self::Message, Self::Metadata>,
    ) {
        todo!()
    }
}

#[derive(Clone)]
pub struct Overall {
    state: (),
}

impl Overall {
    pub fn new() -> Self {
        Self { state: () }
    }
}

impl Leaderboard for Overall {
    type Message = super::sources::SharedMessage;
    type Metadata = super::metadata::Metadata;

    fn update(
        &mut self,
        _: &MetadataAttached<Self::Message, Self::Metadata>,
    ) {
        todo!()
    }
}
