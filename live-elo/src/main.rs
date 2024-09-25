use std::sync::Arc;

use lbo::{performances::StandardLeaderboard, Pipeline};
use normal_leaderboards::{
    exporter::{shared_processor::SharedHandle, DummyExporter, MultiExporter},
    filter::DummyFilter,
    performances::FanoutPerformances,
    scoring::MessageCountScoring,
    sources::{twitch::TwitchMessageSourceHandle, CancellableSource, TokioTaskSource},
};
use tracing::{info, trace};
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

    let cancellation_token = tokio_util::sync::CancellationToken::new();

    let cancellation_signal_task = {
        let cancellation_token = cancellation_token.clone();

        tokio::task::spawn(async move {
            trace!("waiting for ctrl-c signal");
            tokio::signal::ctrl_c().await.ok();
            info!("info ctrl-c signal");
            cancellation_token.cancel();
        })
    };

    let shared_handle = SharedHandle::new(Arc::new(std::collections::HashMap::from([(
        LeaderboardName::new("message_count".to_string()),
        Arc::new(LeaderboardElos::new(Vec::new())),
    )])));

    let websocket_server = normal_leaderboards::exporter::websocket::UnstartedWebsocketServer::new(
        shared_handle.clone(),
    );

    let pipeline = Pipeline::builder()
        .source(CancellableSource::new(
            TokioTaskSource::builder()
                .add_source(TwitchMessageSourceHandle::spawn("ironmouse"))
                .build(),
            cancellation_token,
        ))
        .filter(DummyFilter::new())
        .performances(
            FanoutPerformances::builder()
                .add_performance_processor(StandardLeaderboard::new(
                    MessageCountScoring::new(),
                    MultiExporter::pair(
                        DummyExporter::new(),
                        shared_handle.create_consumer_for_leaderboard(LeaderboardName::new(
                            "message_count".to_string(),
                        )),
                    ),
                ))
                .build(),
        )
        .build();

    let webserver_handle = websocket_server.start().await;
    let pipeline = pipeline.run().await.unwrap();
    tracing::debug!("pipeline finished");
    webserver_handle.close().await;
    pipeline.close().await;
    tracing::debug!("webserver handle finished");

    cancellation_signal_task.abort();
}
