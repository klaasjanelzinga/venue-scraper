use std::env;
use std::sync::Once;
use tracing::Level;

use mongodb::Database;

use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{filter, fmt, prelude::*};
use venue_scraper_api::agenda::create_mongo_connection;
use venue_scraper_api::config::Config;

static LOG_INIT: Once = Once::new();

pub struct TestFixtures {
    pub db: Database,
    pub config: Config,
}

fn set_env_var_if_not_set(env_var: &str, default_value: &str) {
    if env::var(env_var).is_err() {
        env::set_var(env_var, default_value)
    }
}

pub async fn setup() -> TestFixtures {
    LOG_INIT.call_once(|| {
        if env::var_os("RUST_LOG").is_none() {
            tracing_subscriber::registry()
                .with(fmt::layer().compact().with_span_events(FmtSpan::CLOSE))
                .with(
                    filter::Targets::new()
                        .with_target("venue_scraper_api", Level::INFO)
                        .with_target("test_tivoli", Level::INFO)
                        // .with_target("html5ever", Level::INFO)
                        .with_default(Level::WARN),
                )
                .init();
        } else {
            fmt::init();
        }
    });
    set_env_var_if_not_set("ENVIRONMENT", "localhost");
    set_env_var_if_not_set("MONGO_USER", "venue_scraper_test");
    set_env_var_if_not_set("MONGO_PASS", "venue_scraper");
    set_env_var_if_not_set("MONGO_HOST", "localhost");
    set_env_var_if_not_set("MONGO_PORT", "5900");
    set_env_var_if_not_set("MONGO_DB", "venue-scraper-test");
    set_env_var_if_not_set("JWT_TOKEN_SECRET", "venue-scraper-test");

    let config = Config::from_environment();
    let db = create_mongo_connection(&config).await.unwrap();

    TestFixtures { config, db }
}
