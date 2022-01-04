use std::path::PathBuf;
use structopt::StructOpt;

/// Configuration for ProviderMarket actor.
#[derive(StructOpt, Clone, Debug)]
pub struct MarketConfig {
    #[structopt(long, env, default_value = "20.0")]
    pub agreement_events_interval: f32,
    #[structopt(long, env, default_value = "20.0")]
    pub negotiation_events_interval: f32,
    #[structopt(long, env, default_value = "10.0")]
    pub agreement_approve_timeout: f32,
    #[structopt(skip = "you-forgot-to-set-session-id")]
    pub session_id: String,
    #[structopt(long, env, default_value = "negotiations", parse(from_os_str))]
    pub negotiators_workdir: PathBuf,
}
