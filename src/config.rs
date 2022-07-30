use std::env;
use std::fmt::{Display, Formatter};

#[derive(Clone)]
pub struct Config {
    pub mongo_db: String,
    pub mongo_url: String,
    pub masked_mongo_url: String,
    pub environment: String,
    pub application_name: String,
}

fn os_var_as_string(var: &str) -> String {
    env::var_os(var)
        .unwrap_or_else(|| {
            panic!(
                "{}",
                format!("Environment {} not set. Cannot start application", var)
            )
        })
        .into_string()
        .unwrap()
}

impl Config {
    pub fn from_environment() -> Self {
        let mongo_db = os_var_as_string("MONGO_DB");
        let mongo_host = os_var_as_string("MONGO_HOST");
        let mongo_port = os_var_as_string("MONGO_PORT");
        let mongo_user = os_var_as_string("MONGO_USER");
        let mongo_pass = os_var_as_string("MONGO_PASS");

        let environment = os_var_as_string("ENVIRONMENT");
        let mongo_url = format!(
            "mongodb://{}:{}@{}:{}/{}",
            mongo_user, mongo_pass, mongo_host, mongo_port, mongo_db
        );
        let masked_mongo_url = format!(
            "mongodb://{}:******@{}:{}/{}",
            mongo_user, mongo_host, mongo_port, mongo_db
        );

        Config {
            environment,
            mongo_url,
            masked_mongo_url,
            mongo_db,
            application_name: "venue-scraper".to_string(),
        }
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("mongo_db", &self.mongo_db)
            .field("mongo_pass", &self.masked_mongo_url)
            .field("environment", &self.environment)
            .field("mongo_url", &self.masked_mongo_url)
            .field("application_name", &self.application_name)
            .finish()
    }
}
