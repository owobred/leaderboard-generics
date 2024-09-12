use lbo::{performances::StandardLeaderboard, Pipeline};
use normal_leaderboards::{
    exporter::{websocket::LeaderboardName, DummyExporter, MultiExporter},
    filter::DummyFilter,
    scoring::MessageCountScoring,
    sources::TwitchMessageSourceHandle,
};

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

    let websocket_server = normal_leaderboards::exporter::websocket::UnstartedWebsocketServer::new(
        std::collections::HashMap::from([(LeaderboardName::new("message_count"), Vec::new())]),
    );

    let pipeline = Pipeline::builder()
        .source(TwitchMessageSourceHandle::spawn())
        .filter(DummyFilter::new())
        .performances(StandardLeaderboard::new(
            MessageCountScoring::new(),
            MultiExporter::pair(
                DummyExporter::new(),
                websocket_server.get_exporter_for_leaderboard(LeaderboardName::new("message_count")),
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
