use std::fmt::{Debug, Display, Formatter};

use scraper::{ElementRef, Html};
use tokio::join;
use tracing::{error, info, instrument, trace, trace_span, warn};

use errors::ErrorKind;
use http_sender::{HttpSend, Sender};
use parser::CssSelectors;

pub mod errors;
pub mod http_sender;
mod parser;

#[derive(Debug)]
pub struct Venue {
    pub name: String,
}

impl Display for Venue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Venue").field("url", &self.name).finish()
    }
}

#[derive(Debug)]
pub struct Agenda {
    pub title: String,
    pub description: Option<String>,
    pub url: String,
}

impl Display for Agenda {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Agenda")
            .field("url", &self.url)
            .field("title", &self.title)
            .field("description", &self.description)
            .finish()
    }
}

pub struct VenueScraper<'a, HttpSender: HttpSend = Sender> {
    client: &'a reqwest::Client,
    pub http_sender: HttpSender,
    venue: Venue,
    agenda_urls: Vec<String>,
    css_selectors: CssSelectors,
}

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

impl<HttpSender: HttpSend> VenueScraper<'_, HttpSender> {
    pub fn tivoli_with_sender_and_client(
        http_sender: HttpSender,
        client: &reqwest::Client,
    ) -> Result<VenueScraper<HttpSender>, ErrorKind> {
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
        })
    }

    pub fn spot_groningen_with_sender_and_client(
        http_sender: HttpSender,
        client: &reqwest::Client,
    ) -> Result<VenueScraper<HttpSender>, ErrorKind> {
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
            sync_results.total_urls_fetched += 1;
            let mut number_of_agenda_items = 0;

            let body = trace_span!("fetching_url", agenda_url=agenda_url, venue=?self.venue)
                .in_scope(|| async {
                    let request = http_sender::build_request_for_url(&self.client, &agenda_url)?;
                    let response = self.http_sender.send(request).await?;
                    http_sender::body_for_response(response).await
                })
                .await?;

            let parsed_html = Html::parse_document(&body);

            let agenda_res: Vec<Agenda> = parsed_html.select(&self.css_selectors.agenda_item)
                .map(| agenda_item_element | parser::agenda_from_element(&agenda_item_element, &self.css_selectors))
                .filter_map(| it | {
                    number_of_agenda_items += 1;
                    match it {
                        Ok(agenda_item) => Some(agenda_item),
                        Err(err) => {
                            sync_results.total_unparseable_items += 1;
                            warn!("Cannot parse an item {}", err);
                            None
                        }
                    }
                })
                .collect()
                ;

            // upsert to store. Only insert if agenda_item.url does not exist.

            info!("number of results {} {}", sync_results, agenda_res.len());
            sync_results.total_items += number_of_agenda_items;
            needs_next_page = number_of_agenda_items >= number_of_agenda_last_iteration;
            number_of_agenda_last_iteration = number_of_agenda_items;
        }

        info!("Sync completed {} {}", self.venue, sync_results);
        Ok(sync_results)
    }
}

pub async fn sync_venues() -> Result<i64, ErrorKind> {
    trace!("sync_venues");
    let client = reqwest::Client::new();

    let mut tivoli_syncer = VenueScraper::tivoli_with_sender_and_client(Sender, &client).unwrap();
    let mut spot_groningen_syncer =
        VenueScraper::spot_groningen_with_sender_and_client(Sender, &client).unwrap();

    let (spot_result, tivoli_result) = join!(spot_groningen_syncer.sync(), tivoli_syncer.sync());

    for result in [spot_result, tivoli_result] {
        match result {
            Err(err) => error!("Error syncing tivoli {}", err),
            _ => info!("All went well"),
        }
    }
    Ok(3)
}
