use std::env::var_os;
use std::error::Error;

use tracing::info;

use venue_scraper_api::sync_venues;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if var_os("RUST_LOG").is_none() {
        tracing_subscriber::fmt::init();
    } else {
        tracing_subscriber::fmt::init();
    }

    info!("Starting application {}", env!("CARGO_PKG_VERSION"));

    sync_venues().await.expect("ok");

    Ok(())
}
