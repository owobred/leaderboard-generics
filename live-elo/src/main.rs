use std::sync::Arc;

use lbo::{performances::StandardLeaderboard, Pipeline};
use normal_leaderboards::{
    exporter::{shared_processor::SharedHandle, DummyExporter, MultiExporter},
    filter::DummyFilter,
    scoring::MessageCountScoring,
    sources::TwitchMessageSourceHandle,
};
use websocket_shared::{LeaderboardElos, LeaderboardName};

#[tokio::main]
async fn main() {
    {
        use tracing_subscriber::prelude::*;

        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .with_writer(std::io::stderr)
                    .with_filter(tracing_subscriber::EnvFilter::from_default_env()),
            )
            .init();
    }

    let shared_handle = SharedHandle::new(Arc::new(std::collections::HashMap::from([(
        LeaderboardName::new("message_count".to_string()),
        Arc::new(LeaderboardElos::new(Vec::new())),
    )])));

    let websocket_server = normal_leaderboards::exporter::websocket::UnstartedWebsocketServer::new(
        shared_handle.clone(),
    );

    let pipeline = Pipeline::builder()
        .source(TwitchMessageSourceHandle::spawn())
        .filter(DummyFilter::new())
        .performances(StandardLeaderboard::new(
            MessageCountScoring::new(),
            MultiExporter::pair(
                DummyExporter::new(),
                shared_handle.create_consumer_for_leaderboard(LeaderboardName::new(
                    "message_count".to_string(),
                )),
            ),
        ))
        .build();

    let webserver_handle = websocket_server.start().await;
    // tracing::debug!("sleeping");
    // tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    // tracing::debug!("sleep done");
    pipeline.run().await.unwrap();
    tracing::debug!("pipeline finished");
    // tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    webserver_handle.close().await;
    tracing::debug!("webserver handle finished");
    // tokio::time::sleep(std::time::Duration::from_secs(10)).await;
}
