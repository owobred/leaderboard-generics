use elo_lbo::user_impl::{
    filters::OptoutFilter,
    leaderboards::{BitsOnly, Overall},
    metadata::MetadataProcessor,
    metrics::MetricsProcessor,
    sources::TwitchMessageSource,
};
use lbo::{LeaderboardSetBuilder, PipelineBuilder, StaticFilterSet};

fn main() {
    let leaderboard_pipeline = PipelineBuilder::new()
        .source(
            TwitchMessageSource::new(),
            // TODO: add AsyncMpscSource here
        )
        .filter(
            // FilterChainBuilder::new()
            // .add_filter(OptoutFilter::new())
            // .build(),
            StaticFilterSet::new(OptoutFilter::new()),
            // then use `.append(...)` to add more to the static filter set
        )
        .metadata(MetadataProcessor::new())
        .metrics(MetricsProcessor::new())
        .leaderboard(
            LeaderboardSetBuilder::new()
                .add_leaderboard(Overall::new())
                .add_leaderboard(BitsOnly::new())
                .build(),
        )
        .build();

    leaderboard_pipeline.run().unwrap()
}
