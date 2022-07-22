use std::env::var_os;
use std::error::Error;

use log::{info, LevelFilter};
use pretty_env_logger::{formatted_timed_builder, init};
use venue_scraper_api::sync_venues;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if var_os("RUST_LOG").is_none() {
        formatted_timed_builder()
            .filter_module("venue_scraper_api", LevelFilter::Trace)
            .filter_module("venue_scraper", LevelFilter::Trace)
            .filter_level(LevelFilter::Warn)
            .init();
    } else {
        init()
    }

    info!("Starting application {}", env!("CARGO_PKG_VERSION"));

    sync_venues().await.expect("ok");

    Ok(())
}
