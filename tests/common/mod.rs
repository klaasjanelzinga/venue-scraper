use std::env;
use std::sync::Once;
use tracing::Level;

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{filter, fmt, prelude::*};

static LOG_INIT: Once = Once::new();

pub async fn setup() -> () {
    LOG_INIT.call_once(|| {
        if env::var_os("RUST_LOG").is_none() {
            tracing_subscriber::registry()
                .with(fmt::layer().compact())
                .with(
                    filter::Targets::new()
                        .with_target("venue_scraper_api", Level::TRACE)
                        .with_target("test_tivoli", Level::TRACE)
                        // .with_target("html5ever", Level::INFO)
                        .with_default(Level::WARN),
                )
                .init();
        } else {
            fmt::init();
        }
    });
}
