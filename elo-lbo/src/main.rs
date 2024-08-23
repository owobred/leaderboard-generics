use elo_lbo::user_impl::{
    filters::OptoutFilter,
    leaderboards::{BitsOnly, Overall},
    metadata::MetadataProcessor,
    metrics::MetricsProcessor,
    sources::{DiscordMessageSource, TwitchMessageSource},
};
use lbo::{LeaderboardSetBuilder, MessageSourceSetBuilder, PipelineBuilder, StaticFilterSet};

fn main() {
    let leaderboard_pipeline = PipelineBuilder::new()
        .source(
            MessageSourceSetBuilder::new()
                .add_source(TwitchMessageSource::new())
                .add_source(DiscordMessageSource::new())
                .build(),
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
