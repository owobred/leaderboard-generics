use elo_lbo::{
    leaderboardset::AsyncLeaderboardSetBuilder,
    sourceset::AsyncSourcesBuilder,
    user_impl::{
        filters::OptoutFilter,
        leaderboards::{BitsOnly, Overall},
        performance::PerformanceProcessor,
        sources::{DiscordMessageSource, TwitchMessageSource},
    },
};
use lbo::{PipelineBuilder, StaticFilterSet};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let (sources, sources_handle) = AsyncSourcesBuilder::new()
        .spawn_source(TwitchMessageSource::new())
        .spawn_source(DiscordMessageSource::new())
        .build();

    let (leaderboards, leaderboards_handle) = AsyncLeaderboardSetBuilder::new()
        .spawn_leaderboard(Overall::new())
        .spawn_leaderboard(BitsOnly::new())
        .build();

    let leaderboard_pipeline = PipelineBuilder::new()
        .source(sources)
        .filter(
            // One option is to use the following, which uses
            // dynamic dispatch for the filter chain
            // FilterChainBuilder::new()
            // .add_filter(OptoutFilter::new())
            // .build(),
            // Alternatively, you can use [StaticFilterSet] to use
            // static dispatch
            StaticFilterSet::new(OptoutFilter::new()),
            // then use `.append(...)` to add more to the static filter set
        )
        .performance(PerformanceProcessor::new())
        .leaderboard(leaderboards)
        .build();

    leaderboard_pipeline.run().unwrap();
    sources_handle.join().await;
    leaderboards_handle.join().await;
}
