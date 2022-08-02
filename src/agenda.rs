use crate::{Config, ErrorKind};
use mongodb::bson::doc;
use mongodb::bson::Bson;
use mongodb::{Client, Collection, Cursor, Database};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use tracing::{info, trace};

use mongodb::options::ClientOptions;

#[derive(Debug)]
pub struct Venue {
    pub venue_id: String,
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

    pub url: String,
    pub title: String,
    pub description: Option<String>,

    pub venue_id: String,

    pub needs_details: bool,
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

/// Retrieve an Agenda by the url.
pub async fn get_agenda_by_url(url: &str, db: &Database) -> Result<Option<Agenda>, ErrorKind> {
    let agenda_collection = agenda_collection(&db);
    let optional_agenda = agenda_collection.find_one(doc! {"url": &url}, None).await?;
    match optional_agenda {
        Some(agenda) => Ok(Some(agenda)),
        None => Ok(None),
    }
}

pub struct UpsertAgendaResult {
    pub agenda: Agenda,
    pub inserted: bool,
}

pub async fn insert_or_get_agenda(
    agenda: &Agenda,
    db: &Database,
) -> Result<UpsertAgendaResult, ErrorKind> {
    let agenda_collection = agenda_collection(&db);
    match get_agenda_by_url(&agenda.url, &db).await? {
        Some(agenda) => Ok(UpsertAgendaResult {
            agenda,
            inserted: false,
        }),
        None => {
            let new_agenda = Agenda {
                _id: None,
                venue_id: agenda.venue_id.clone(),
                url: agenda.url.clone(),
                description: agenda.description.clone(),
                title: agenda.title.clone(),
                needs_details: true,
            };
            let _insert_result = agenda_collection.insert_one(&new_agenda, None).await?;

            Ok(UpsertAgendaResult {
                agenda: new_agenda,
                inserted: true,
            })
        }
    }
}

pub async fn update_agenda(agenda: &Agenda, db: &Database) -> Result<(), ErrorKind> {
    let collection = agenda_collection(&db);
    let update_results = collection
        .replace_one(doc! { "url": &agenda.url }, agenda, None)
        .await?;

    if update_results.matched_count != 1 || update_results.modified_count != 1 {
        Err(ErrorKind::GenericError)
    } else {
        Ok(())
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

pub async fn execute_on_agenda_items(
    venue_id: &String,
    db: &Database,
) -> Result<Cursor<Agenda>, ErrorKind> {
    let collection = agenda_collection(db);
    let filter = doc! { "venue_id": venue_id };
    let cursor = collection.find(filter, None).await?;
    Ok(cursor)
}
