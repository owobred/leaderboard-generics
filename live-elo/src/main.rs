use lbo::{performances::StandardLeaderboard, Pipeline};
use normal_leaderboards::{
    exporter::{websocket::LeaderboardName, DummyExporter, MultiExporter},
    filter::DummyFilter,
    scoring::DummyScoring,
    sources::DummyTwitchSource,
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
        std::collections::HashMap::from([(LeaderboardName::new("dummy"), Vec::new())]),
    );

    let pipeline = Pipeline::builder()
        .source(DummyTwitchSource::new())
        .filter(DummyFilter::new())
        .performances(StandardLeaderboard::new(
            DummyScoring::new(),
            MultiExporter::pair(
                DummyExporter::new(),
                websocket_server.get_exporter_for_leaderboard(LeaderboardName::new("dummy")),
            ),
        ))
        .build();

    let webserver_handle = websocket_server.start().await;
    tracing::debug!("sleeping");
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    tracing::debug!("sleep done");
    pipeline.run().await.unwrap();
    tracing::debug!("pipeline finished");
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    webserver_handle.close().await;
    tracing::debug!("webserver handle finished");
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
}
