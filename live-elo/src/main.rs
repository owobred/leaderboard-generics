use lbo::{performances::StandardLeaderboard, Pipeline};
use normal_leaderboards::{
    exporter::{DummyExporter, MultiExporter},
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
    tracing::info!("erm");
    // let (websocket_server_join, websocket_server) =
    //     normal_leaderboards::exporter::websocket::UnloadedLeaderboards::builder()
    //         .add_leaderboard(LeaderboardDescriptor::new("dummy", "./dummy.bin"))
    //         .build()
    //         .;

    let websocket_server =
        normal_leaderboards::exporter::websocket::UnstartedWebsocketServer::new();

    let pipeline = Pipeline::builder()
        .source(DummyTwitchSource::new())
        .filter(DummyFilter::new())
        .performances(StandardLeaderboard::new(
            DummyScoring::new(),
            MultiExporter::pair(
                DummyExporter::new(),
                websocket_server.get_exporter_for_leaderboard("dummy".to_string()),
            ),
        ))
        .build();

    let webserver_handle = websocket_server
        .start(std::collections::HashMap::from([(
            "dummy".to_string(),
            Vec::new(),
        )]))
        .await;
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
