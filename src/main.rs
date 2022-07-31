use std::env::var_os;
use std::error::Error;
use std::rc::Rc;

use tracing::info;
use venue_scraper_api::agenda::create_mongo_connection;
use venue_scraper_api::config::Config;
use venue_scraper_api::http_sender::DefaultHttpSender;

use venue_scraper_api::sync_venues;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if var_os("RUST_LOG").is_none() {
        tracing_subscriber::fmt::init();
    } else {
        tracing_subscriber::fmt::init();
    }

    info!("Starting application {}", env!("CARGO_PKG_VERSION"));
    let config = Config::from_environment();
    let client = reqwest::Client::new();
    let db = create_mongo_connection(&config).await?;
    let http_sender = Rc::new(DefaultHttpSender);

    info!("Start sync of the venues");
    let sync_results = sync_venues(&client, &db, http_sender).await?;
    info!("Sync results of the venues {}", sync_results);

    Ok(())
}
