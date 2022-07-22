use std::env;
use std::sync::Once;
use log::LevelFilter;

static LOG_INIT: Once = Once::new();

pub async fn setup() -> () {
    LOG_INIT.call_once(|| {
        if env::var_os("RUST_LOG").is_none() {
            pretty_env_logger::formatted_timed_builder()
                .filter_module("venue_scraper", LevelFilter::Trace)
                .filter_module("test_tivoli", LevelFilter::Debug)
                .filter_level(LevelFilter::Warn)
                .init();
        } else {
            pretty_env_logger::init();
        }
    });
}
