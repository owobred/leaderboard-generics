use lbo::{
    user_impl::{
        filters::OptoutFilter,
        leaderboards::{BitsOnly, Overall},
        metadata::MetadataProcessor,
        metrics::MetricsProcessor,
        sources::{DiscordMessageSource, TwitchMessageSource},
    },
    FilterChainBuilder, LeaderboardSetBuilder, MessageSourceSetBuilder, PipelineBuilder,
};

fn main() {
    let leaderboard_pipeline = PipelineBuilder::new()
        .source(
            MessageSourceSetBuilder::new()
                .add_source(TwitchMessageSource::new())
                .add_source(DiscordMessageSource::new())
                .build(),
        )
        .filter(
            FilterChainBuilder::new()
                .add_filter(OptoutFilter::new())
                .build(),
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
}
