use lbo::{Leaderboard, PerformanceAttached};

#[derive(Clone)]
pub struct BitsOnly;

impl BitsOnly {
    pub fn new() -> Self {
        Self {}
    }
}

impl Leaderboard for BitsOnly {
    type Message = super::sources::SharedMessage;
    type Performance = super::performance::Performance;

    fn update(&mut self, _: &PerformanceAttached<Self::Message, Self::Performance>) {
        todo!()
    }
}

#[derive(Clone)]
pub struct Overall;

impl Overall {
    pub fn new() -> Self {
        Self {}
    }
}

impl Leaderboard for Overall {
    type Message = super::sources::SharedMessage;
    type Performance = super::performance::Performance;

    fn update(&mut self, _: &PerformanceAttached<Self::Message, Self::Performance>) {
        todo!()
    }
}
