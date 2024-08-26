use lbo::{Leaderboard, PerformanceAttached};

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
    type Performance = super::performance::Performance;

    fn update(&mut self, _: &PerformanceAttached<Self::Message, Self::Performance>) {
        todo!()
    }
}

impl Exportable for BitsOnly {
    type State = ();

    fn name(&self) -> String {
        "bits_only".to_string()
    }

    fn get_state(&self) -> &Self::State {
        &self.state
    }

    fn get_state_mut(&mut self) -> &mut Self::State {
        &mut self.state
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
    type Performance = super::performance::Performance;

    fn update(&mut self, _: &PerformanceAttached<Self::Message, Self::Performance>) {
        todo!()
    }
}

impl Exportable for Overall {
    type State = ();

    fn name(&self) -> String {
        "overall".to_string()
    }

    fn get_state(&self) -> &Self::State {
        &self.state
    }

    fn get_state_mut(&mut self) -> &mut Self::State {
        &mut self.state
    }
}

pub trait Exportable {
    type State;

    // TODO: this might be better as an `&'static str`?
    fn name(&self) -> String;

    fn get_state(&self) -> &Self::State;
    fn get_state_mut(&mut self) -> &mut Self::State;
}
