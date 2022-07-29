use std::env;
use std::sync::Once;
use tracing::Level;

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{filter, fmt, prelude::*};
use tracing_subscriber::fmt::format::FmtSpan;

static LOG_INIT: Once = Once::new();

pub async fn setup() -> () {
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
}
