use crate::{Config, ErrorKind};
use mongodb::{Client, Collection, Database};
use std::fmt::{Display, Formatter};
use tracing::{info, trace};
use mongodb::bson::doc;
use mongodb::bson::Bson;
use serde::{Deserialize, Serialize};

use mongodb::options::ClientOptions;

#[derive(Debug)]
pub struct Venue {
    pub name: String,
}

impl Display for Venue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Venue").field("url", &self.name).finish()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Agenda {
    #[serde(skip_serializing)]
    pub _id: Option<Bson>,
    pub title: String,
    pub description: Option<String>,
    pub url: String,
}

impl Display for Agenda {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Agenda")
            .field("_id", &self._id)
            .field("url", &self.url)
            .field("title", &self.title)
            .field("description", &self.description)
            .finish()
    }
}

/// Creates the agenda collection for the database.
fn agenda_collection(db: &Database) -> Collection<Agenda> {
    db.collection::<Agenda>("agenda")
}

pub async fn get_agenda_by_url(url: &str, db: &Database) -> Result<Option<Agenda>, ErrorKind> {
    let agenda_collection = agenda_collection(&db);
    let optional_agenda = agenda_collection.find_one(doc! {"url": &url}, None).await?;
    match optional_agenda {
        Some(agenda) => Ok(Some(agenda)),
        None => Ok(None)
    }
}

pub async fn upsert_agenda(agenda: &Agenda, db: &Database) -> Result<Agenda, ErrorKind> {
    let agenda_collection = agenda_collection(&db);
    match get_agenda_by_url(&agenda.url, &db).await? {
        Some(agenda) => Ok(agenda),
        None => {
            let insert_result = agenda_collection.insert_one(agenda, None).await?;
            trace!("Inserted a new agenda");
            Ok(Agenda {
                _id: None,
                url: agenda.url.clone(),
                description: agenda.description.clone(),
                title: agenda.title.clone(),
            })
        }
    }
}

pub async fn create_mongo_connection(config: &Config) -> Result<Database, ErrorKind> {
    trace!("Connecting mongodb, config {}", config);
    let client_options = ClientOptions::parse(&config.mongo_url).await?;
    let client = Client::with_options(client_options)?;
    info!("Mongo db client connected with config {}", config);
    let db = client.database(&config.mongo_db);
    Ok(db)
}
