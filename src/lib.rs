use std::fmt::{Debug, Display, Formatter};
use std::future::Future;

use agenda::{Agenda, Venue};
use scraper::Html;
use tokio::{join};
use tracing::{error, info, trace, trace_span, warn};

use crate::agenda::{create_mongo_connection, upsert_agenda};
use crate::config::Config;
use errors::ErrorKind;
use http_sender::{DefaultHttpSender, HttpSender};
use mongodb::Database;
use parser::CssSelectors;

pub mod agenda;
pub mod config;
pub mod errors;
pub mod http_sender;
mod parser;

#[derive(Debug)]
pub struct SyncingResult {
    pub total_urls_fetched: u32,
    pub total_items: u32,
    pub total_unparseable_items: u32,
}

impl Display for SyncingResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyncingResults")
            .field("total_urls_fetched", &self.total_urls_fetched)
            .field("total_items", &self.total_items)
            .field("total_unparseable_items", &self.total_unparseable_items)
            .finish()
    }
}

pub struct VenueScraper {
    client: reqwest::Client,
    pub http_sender: Box<dyn HttpSender>,
    venue: Venue,
    agenda_urls: Vec<String>,
    css_selectors: CssSelectors,
    db: Database,
}

impl VenueScraper {
    pub fn tivoli_with_sender_and_client(
        http_sender: Box<dyn HttpSender>,
        client: reqwest::Client,
        db: Database,
    ) -> Result<VenueScraper, ErrorKind> {
        let mut agenda_urls = Vec::new();
        agenda_urls.push(String::from("https://www.tivolivredenburg.nl/agenda/"));
        for count in 2..20 {
            agenda_urls.push(format!(
                "https://www.tivolivredenburg.nl/agenda/page/{}/",
                count
            ));
        }
        let agenda_item = parser::selector_for(r#"li.agenda-list-item"#)?;
        let url = parser::selector_for(r#"a.agenda-list-item__title-link"#)?;
        let title = parser::selector_for(r#"a.agenda-list-item__title-link"#)?;
        let description = parser::selector_for(r#"p.agenda-list-item__text"#)?;

        let css_selectors = CssSelectors {
            agenda_item,
            url,
            title,
            description,
        };

        let venue = Venue {
            name: "Tivoli Utrecht".to_string(),
        };

        Ok(VenueScraper {
            client,
            http_sender,
            agenda_urls,
            venue,
            css_selectors,
            db,
        })
    }

    pub fn spot_groningen_with_sender_and_client(
        http_sender: Box<dyn HttpSender>,
        client: reqwest::Client,
        db: Database,
    ) -> Result<VenueScraper, ErrorKind> {
        let mut agenda_urls = Vec::new();
        agenda_urls.push(String::from("https://www.spotgroningen.nl/programma/"));

        let agenda_item = parser::selector_for(r#"article.program__item"#)?;
        let url = parser::selector_for(r#"a.program__link"#)?;
        let title = parser::selector_for(r#"h1"#)?;
        let description = parser::selector_for(r#"p"#)?;

        let css_selectors = CssSelectors {
            agenda_item,
            url,
            title,
            description,
        };

        let venue = Venue {
            name: "Spot Groningen".to_string(),
        };

        Ok(VenueScraper {
            client,
            http_sender,
            agenda_urls,
            venue,
            css_selectors,
            db,
        })
    }

    pub async fn sync(&mut self) -> Result<SyncingResult, ErrorKind> {
        info!("Syncing venue {}", self.venue);
        let mut number_of_agenda_last_iteration = 0;
        let mut needs_next_page = true;
        let mut sync_results = SyncingResult {
            total_urls_fetched: 0,
            total_items: 0,
            total_unparseable_items: 0,
        };

        for agenda_url in self.agenda_urls.iter() {
            if !needs_next_page {
                continue;
            }
            let mut number_of_agenda_items = 0;
            let mut number_of_unparseable_agenda_items = 0;

            let body = trace_span!("fetching_url", agenda_url=agenda_url, venue=?self.venue)
                .in_scope(|| async {
                    let request = http_sender::build_request_for_url(&self.client, &agenda_url)?;
                    let response = self.http_sender.send(request).await?;
                    http_sender::body_for_response(response).await
                })
                .await?;

            let parsed_html =
                trace_span!("parsing_document").in_scope(|| Html::parse_document(&body));

            let agenda_res: Vec<Agenda> = trace_span!("doc_to_agenda_items").in_scope(|| {
                parsed_html
                    .select(&self.css_selectors.agenda_item)
                    .map(|agenda_item_element| {
                        parser::agenda_from_element(&agenda_item_element, &self.css_selectors)
                    })
                    .filter_map(|it| {
                        number_of_agenda_items += 1;
                        match it {
                            Ok(agenda_item) => Some(agenda_item),
                            Err(err) => {
                                number_of_unparseable_agenda_items += 1;
                                warn!("Cannot parse an item {}", err);
                                None
                            }
                        }
                    })
                    .collect()
            });

            // upsert to store. Only insert if agenda_item.url does not exist.
            for agenda in agenda_res {
                upsert_agenda(&agenda, &self.db).await?;
            }

            sync_results.total_items += number_of_agenda_items;
            sync_results.total_urls_fetched += 1;
            sync_results.total_unparseable_items += number_of_unparseable_agenda_items;

            needs_next_page = number_of_agenda_items >= number_of_agenda_last_iteration;
            number_of_agenda_last_iteration = number_of_agenda_items;
            info!("number of results {}", sync_results);
        }

        info!("Sync completed {} {}", self.venue, sync_results);
        Ok(sync_results)
    }
}

pub async fn sync_venues() -> Result<i64, ErrorKind> {
    trace!("sync_venues");
    let config = Config::from_environment();
    let client = reqwest::Client::new();
    let db = create_mongo_connection(&config).await?;

    let mut tivoli_syncer = VenueScraper::tivoli_with_sender_and_client(
        Box::new(DefaultHttpSender),
        client.clone(),
        db.clone(),
    )
    .unwrap();
    let mut spot_groningen_syncer = VenueScraper::spot_groningen_with_sender_and_client(
        Box::new(DefaultHttpSender),
        client.clone(),
        db.clone(),
    )
    .unwrap();

    let (spot_result, tivoli_result) = join!(spot_groningen_syncer.sync(), tivoli_syncer.sync());

    for result in [spot_result, tivoli_result] {
        match result {
            Err(err) => error!("Error syncing tivoli {}", err),
            _ => info!("All went well"),
        }
    }
    Ok(3)
}
