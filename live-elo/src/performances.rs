pub struct NullPerformanceAttacher<M> {
    _phantom: std::marker::PhantomData<M>,
}

impl<M> NullPerformanceAttacher<M> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<M> lbo::PerformanceAttacher for NullPerformanceAttacher<M> {
    type Message = M;
    type Performance = ();

    fn attach_performance(
        &self,
        message: Self::Message,
    ) -> lbo::PerformanceAttached<Self::Message, Self::Performance> {
        lbo::PerformanceAttached {
            message,
            performance: (),
        }
    }
}
