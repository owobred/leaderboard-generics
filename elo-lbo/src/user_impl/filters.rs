use lbo::Filter;

pub struct OptoutFilter;

impl Filter for OptoutFilter {
    type Message = super::sources::SharedMessage;

    fn keep_message(&self, _: &Self::Message) -> bool {
        todo!()
    }
}

impl OptoutFilter {
    pub fn new() -> Self {
        Self {}
    }
}
