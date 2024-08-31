use lbo::PipelineBuilder;
use tracing::info;

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

    let twitch_source = live_elo::sources::twitch::SourceBuilder::default()
        .channel_name("vedal987".to_string())
        .run();
    let twitch_source_abort = twitch_source.get_abort_handle();

    let (sources, sources_handle) = elo_lbo::sourceset::AsyncSourcesBuilder::new()
        .spawn_source(twitch_source)
        .build();

    info!("starting pipeline");
    PipelineBuilder::new()
        .source(sources)
        .filter(lbo::NullFilter::new())
        .performance(live_elo::performances::NullPerformanceAttacher::new())
        .leaderboard(live_elo::leaderboards::NullLeaderboard::new())
        .build()
        .run()
        .await
        .unwrap();

    info!("pipeline finished");

    info!("waiting for sources to exit");
    twitch_source_abort.abort();
    sources_handle.join().await;
    info!("done");
}
