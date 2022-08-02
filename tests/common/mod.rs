use std::env;
use std::sync::Once;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, Level};

use mongodb::Database;

use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{filter, fmt, prelude::*};
use venue_scraper_api::agenda::{create_mongo_connection, Agenda};
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
                        .with_target("test_fetch_details", Level::INFO)
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

    empty_users_collection(&db).await;

    TestFixtures { config, db }
}

static mut EMPTY_COLLECTION_BARRIER: u32 = 1;
static mut EMPTIED_COLLECTION_BARRIER: u32 = 0;

pub async fn empty_users_collection(db: &Database) {
    unsafe {
        if EMPTY_COLLECTION_BARRIER == 1 {
            EMPTY_COLLECTION_BARRIER = 0;
            info!("Emptying the collection");
            db.collection::<Agenda>("agenda").drop(None).await.unwrap();
            info!("Emptied the collection");
            EMPTIED_COLLECTION_BARRIER = 1;
        }

        let mut wait_counter = 0;

        while EMPTIED_COLLECTION_BARRIER == 0 {
            info!("Waiting on the emptying of the collection");
            sleep(Duration::from_millis(200)).await;
            wait_counter += 1;

            if wait_counter > 100 {
                assert!(false)
            }
        }
    }

    ()
}
