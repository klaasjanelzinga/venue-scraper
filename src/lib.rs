use futures::stream::TryStreamExt;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

use agenda::Venue;
use scraper::Html;
use tracing::{error, info, trace, trace_span, warn};

use crate::agenda::{execute_on_agenda_items, insert_or_get_agenda, update_agenda, Agenda};
use crate::config::Config;
use crate::http_sender::get_body_for_url;
use errors::ErrorKind;
use http_sender::HttpSender;
use mongodb::Database;
use parser::CssSelectors;
use reqwest::Client;
use tokio::join;

pub mod agenda;
pub mod config;
pub mod errors;
pub mod http_sender;
mod parser;

#[derive(Debug)]
pub struct SyncingResult {
    pub total_urls_fetched: u32,
    pub total_urls_unfetchable: u32,
    pub total_items: u32,
    pub total_unparseable_items: u32,
    pub total_items_inserted: u32,
    pub total_items_updated: u32,
}

impl SyncingResult {
    fn with_zeroes() -> SyncingResult {
        SyncingResult {
            total_items: 0,
            total_items_inserted: 0,
            total_items_updated: 0,
            total_urls_fetched: 0,
            total_urls_unfetchable: 0,
            total_unparseable_items: 0,
        }
    }

    fn add(&mut self, other: &SyncingResult) {
        self.total_unparseable_items += other.total_unparseable_items;
        self.total_items_inserted += other.total_items_inserted;
        self.total_items += other.total_items;
        self.total_urls_fetched += other.total_urls_fetched;
        self.total_items_updated += other.total_items_updated;
        self.total_urls_unfetchable += other.total_urls_unfetchable;
    }
}

impl Display for SyncingResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyncingResults")
            .field("total_urls_fetched", &self.total_urls_fetched)
            .field("total_urls_unfetchable", &self.total_urls_unfetchable)
            .field("total_items", &self.total_items)
            .field("total_unparseable_items", &self.total_unparseable_items)
            .field("total_items_inserted", &self.total_items_inserted)
            .field("total_items_updated", &self.total_items_updated)
            .finish()
    }
}

pub struct VenueScraper {
    client: Client,
    pub http_sender: Rc<dyn HttpSender>,
    venue: Venue,
    agenda_urls: Vec<String>,
    css_selectors: CssSelectors,
    db: Database,
}

impl VenueScraper {
    pub fn tivoli_with_sender_and_client(
        http_sender: Rc<dyn HttpSender>,
        client: Client,
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
            venue_id: "tivoli_utrecht".to_string(),
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
        http_sender: Rc<dyn HttpSender>,
        client: Client,
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
            venue_id: "spot_groningen".to_string(),
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

    pub async fn sync(&self) -> Result<SyncingResult, ErrorKind> {
        info!("Syncing venue {}", self.venue);
        let mut number_of_agenda_last_iteration = 0;
        let mut needs_next_page = true;
        let mut sync_results = SyncingResult::with_zeroes();

        for agenda_url in self.agenda_urls.iter() {
            if !needs_next_page {
                continue;
            }
            let mut number_of_agenda_items = 0;
            let mut number_of_unparseable_agenda_items = 0;

            let body = trace_span!("fetching_url", agenda_url=agenda_url, venue=?self.venue)
                .in_scope(|| async {
                    get_body_for_url(&self.client, &self.http_sender, &agenda_url).await
                })
                .await?;

            let parsed_html =
                trace_span!("parsing_document").in_scope(|| Html::parse_document(&body));

            let agenda_res = trace_span!("doc_to_agenda_items").in_scope(|| {
                parsed_html
                    .select(&self.css_selectors.agenda_item)
                    .map(|agenda_item_element| {
                        parser::agenda_from_element(
                            &agenda_item_element,
                            &self.css_selectors,
                            &self.venue.venue_id,
                        )
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
                    .into_iter()
            });

            trace_span!("store_agenda_items")
                .in_scope(|| async {
                    for agenda in agenda_res {
                        let nw_agenda = insert_or_get_agenda(&agenda, &self.db).await;
                        match nw_agenda {
                            Ok(nw_agenda_result) => {
                                if nw_agenda_result.inserted {
                                    sync_results.total_items_inserted += 1;
                                }
                            }
                            Err(_) => {}
                        }
                    }
                })
                .await;

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

    pub async fn sync_details(&self) -> Result<SyncingResult, ErrorKind> {
        // fetch Agenda items that need details from store.
        let mut sync_results = SyncingResult::with_zeroes();

        let mut cursor = execute_on_agenda_items(&self.venue.venue_id, &self.db).await?;
        while let Some(mut agenda) = cursor.try_next().await? {
            if !agenda.needs_details {
                continue;
            }
            sync_results.total_urls_fetched += 1;
            let details_body = get_body_for_url(&self.client, &self.http_sender, &agenda.url).await;
            match details_body {
                Ok(body) => {
                    let html_document = Html::parse_document(&body);

                    // update other fields.

                    agenda.needs_details = false;
                    match update_agenda(&agenda, &self.db).await {
                        Ok(_) => sync_results.total_items_updated += 1,
                        _ => (),
                    };
                }
                Err(err) => {
                    sync_results.total_urls_unfetchable += 1;
                    info!("Cannot fetch: {}", err);
                }
            }
        }

        info!("Details sync completed, results {}", sync_results);
        Ok(sync_results)
    }
}

pub async fn sync_venues(
    client: &Client,
    db: &Database,
    http_sender: Rc<dyn HttpSender>,
) -> Result<SyncingResult, ErrorKind> {
    trace!("sync_venues");

    let tivoli_syncer = VenueScraper::tivoli_with_sender_and_client(
        Rc::clone(&http_sender),
        client.clone(),
        db.clone(),
    )?;
    let spot_groningen_syncer = VenueScraper::spot_groningen_with_sender_and_client(
        Rc::clone(&http_sender),
        client.clone(),
        db.clone(),
    )?;

    let mut sync_results = SyncingResult::with_zeroes();
    let (spot_result, tivoli_result) = join!(spot_groningen_syncer.sync(), tivoli_syncer.sync());

    for result in [spot_result, tivoli_result] {
        match result {
            Err(err) => error!("Error syncing venue {}", err),
            Ok(results) => {
                sync_results.add(&results);
            }
        }
    }

    let details_result = spot_groningen_syncer.sync_details().await;
    match details_result {
        Ok(results) => sync_results.add(&results),
        Err(err) => error!("Error syncing ddetails {}", err),
    }

    info!("Total {}", sync_results);
    Ok(sync_results)
}
