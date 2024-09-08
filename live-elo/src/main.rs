use std::sync::Arc;

use lbo::{performances::StandardLeaderboard, Pipeline};
use normal_leaderboards::{
    exporter::DummyExporter, filter::DummyFilter, scoring::DummyScoring, sources::DummyTwitchSource,
};

#[tokio::main]
async fn main() {
    let pipeline = Pipeline::builder()
        .source(DummyTwitchSource::new())
        .filter(DummyFilter::new())
        .performances(StandardLeaderboard::new(
            DummyScoring::new(),
            DummyExporter::new(),
        ))
        .build();

    pipeline.run().await.unwrap();
}
