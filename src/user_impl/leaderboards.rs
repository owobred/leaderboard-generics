use crate::Leaderboard;

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
    type Metrics = super::metrics::Metrics;

    fn update(
        &mut self,
        performance: &crate::MetricsAttached<Self::Message, Self::Metadata, Self::Metrics>,
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
    type Metrics = super::metrics::Metrics;

    fn update(
        &mut self,
        performance: &crate::MetricsAttached<Self::Message, Self::Metadata, Self::Metrics>,
    ) {
        todo!()
    }
}
