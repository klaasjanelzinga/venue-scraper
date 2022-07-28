use std::fmt::{Debug, Display, Formatter};

use async_trait::async_trait;
use reqwest::{RequestBuilder, Response, Url};
use scraper::{ElementRef, Html, Selector};
use tracing::{error, info, instrument, span, trace, warn, Level};

use errors::ErrorKind;

pub mod errors;

#[derive(Debug)]
pub struct Agenda {
    title: String,
    description: Option<String>,
    url: String,
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

#[async_trait]
pub trait HttpSend {
    async fn send(&mut self, request: RequestBuilder) -> Result<Response, ErrorKind>;
}

pub struct Sender;

#[async_trait]
impl HttpSend for Sender {
    async fn send(&mut self, request: reqwest::RequestBuilder) -> Result<Response, ErrorKind> {
        let response = request.send().await?;
        Ok(response)
    }
}

pub struct TivoliSyncer<'a, HttpSender: HttpSend = Sender> {
    client: &'a reqwest::Client,
    pub http_sender: HttpSender,
    agenda_urls: Vec<String>,
}

#[instrument(skip(client))]
async fn request_for_url(client: &reqwest::Client, url: &str) -> Result<RequestBuilder, ErrorKind> {
    trace!("request_for_url");
    let url = Url::parse(url)?;
    let request = client.get(url);
    Ok(request)
}

async fn body_for_response(response: reqwest::Response) -> Result<String, ErrorKind> {
    if !response.status().is_success() {
        return Err(ErrorKind::StatusCodeFromUrl {
            status: response.status().to_string(),
            status_code: response.status().as_u16(),
            url: response.url().to_string(),
        });
    }
    let body = response.text().await?;
    Ok(body)
}

fn selector_for(selector: &str) -> Result<Selector, ErrorKind> {
    match Selector::parse(&selector) {
        Ok(selector) => Ok(selector),
        Err(parse_error) => Err(ErrorKind::CssSelectorError {
            message: format!(
                "At line {}, col {}",
                parse_error.location.line, parse_error.location.column
            ),
        }),
    }
}

fn get_text_for_single(text_element: &ElementRef) -> Result<String, ErrorKind> {
    if text_element.text().count() != 1 {
        return Err(ErrorKind::GenericError);
    }
    let text = text_element.text().next().unwrap();
    Ok(text.trim().to_string())
}

fn get_select_on_element<'a>(
    search_in_element: &ElementRef<'a>,
    selector: &Selector,
) -> Result<ElementRef<'a>, ErrorKind> {
    let selected = search_in_element.select(&selector);
    if selected.count() != 1 {
        return Err(ErrorKind::CannotFindSelector {
            selector: format!("{:#?}", selector),
        });
    }
    Ok(search_in_element.select(selector).next().unwrap())
}

fn get_text_from_element(search_in: &ElementRef, selector: &Selector) -> Result<String, ErrorKind> {
    let selected = get_select_on_element(&search_in, &selector)?;
    get_text_for_single(&selected)
}

fn optional_text_from_element(
    search_in: &ElementRef,
    selector: &Selector,
) -> Result<Option<String>, ErrorKind> {
    let selected_result = get_select_on_element(&search_in, &selector);
    match selected_result {
        Ok(selected) => match get_text_for_single(&selected) {
            Ok(text) => Ok(Some(text)),
            Err(err) => Err(err),
        },
        Err(ErrorKind::CannotFindSelector { selector: _ }) => Ok(None),
        Err(err) => Err(err),
    }
}

fn get_text_from_attr(
    search_in: &ElementRef,
    selector: &Selector,
    attr_name: &str,
) -> Result<String, ErrorKind> {
    let selected_element = get_select_on_element(&search_in, &selector)?;
    let attr = selected_element.value().attr(&attr_name);
    match attr {
        Some(value) => Ok(value.to_string()),
        None => Err(ErrorKind::CannotFindAttribute {
            attribute_name: attr_name.to_string(),
        }),
    }
}

fn agenda_from_element(
    element: &ElementRef,
    tivoli_css_selectors: &TivoliCssSelectors,
) -> Result<Agenda, ErrorKind> {
    let url = get_text_from_attr(&element, &tivoli_css_selectors.url, "href")?;
    let title = get_text_from_element(&element, &tivoli_css_selectors.title)?;
    let description = optional_text_from_element(&element, &tivoli_css_selectors.description)?;

    Ok(Agenda {
        title,
        description,
        url: url.to_string(),
    })
}

#[derive(Debug)]
struct TivoliCssSelectors {
    agenda_item: Selector,
    title: Selector,
    url: Selector,
    description: Selector,
}

impl Display for TivoliCssSelectors {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TivoliCssSelectors").finish()
    }
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

impl<HttpSender: HttpSend> TivoliSyncer<'_, HttpSender> {
    pub fn with_sender_and_client(
        http_sender: HttpSender,
        client: &reqwest::Client,
    ) -> TivoliSyncer<HttpSender> {
        let mut agenda_urls = Vec::new();
        agenda_urls.push(String::from("https://www.tivolivredenburg.nl/agenda/"));
        for count in 2..20 {
            agenda_urls.push(format!(
                "https://www.tivolivredenburg.nl/agenda/page/{}/",
                count
            ));
        }

        TivoliSyncer {
            client,
            http_sender,
            agenda_urls,
        }
    }

    pub async fn sync(&mut self) -> Result<SyncingResult, ErrorKind> {
        let tivoli_css_sectors = TivoliCssSelectors {
            agenda_item: selector_for(r#"li.agenda-list-item"#)?,
            url: selector_for(r#"a.agenda-list-item__title-link"#)?,
            title: selector_for(r#"a.agenda-list-item__title-link"#)?,
            description: selector_for(r#"p.agenda-list-item__text"#)?,
        };

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

            span!(Level::INFO, "agenda_url_parsing").in_scope(|| {});

            sync_results.total_urls_fetched += 1;

            info!(event = "Tivoli syncing on the url.", url = agenda_url);
            let request = request_for_url(&self.client, &agenda_url).await?;
            let response = self.http_sender.send(request).await?;
            let body = body_for_response(response).await?;

            let parsed_html = Html::parse_document(&body);

            // let agenda_items = parsed_html
            //     .select(&tivoli_css_sectors.agenda_item)
            //     .map(|element| agenda_from_element(&element, &tivoli_css_sectors))
            //     .filter(|agenda_result| agenda_result.is_ok())
            //     .map(|agenda_result| agenda_result.unwrap());

            for element in parsed_html.select(&tivoli_css_sectors.agenda_item) {
                match agenda_from_element(&element, &tivoli_css_sectors) {
                    Ok(_agenda) => {
                        sync_results.total_items += 1;
                    }
                    Err(ErrorKind::CannotFindSelector { selector }) => {
                        warn!("Cannot locate selector {} in {}", selector, element.html());
                        sync_results.total_unparseable_items += 1;
                        continue;
                    }
                    Err(ErrorKind::CannotFindAttribute { attribute_name }) => {
                        warn!("Cannot locate an attribute {}", attribute_name);
                        sync_results.total_unparseable_items += 1;
                        continue;
                    }
                    Err(err) => return Err(err),
                }
            }

            let number_of_agenda = parsed_html.select(&tivoli_css_sectors.agenda_item).count();
            needs_next_page = number_of_agenda >= number_of_agenda_last_iteration;
            number_of_agenda_last_iteration = number_of_agenda;
        }

        info!(event="Tivoli is synced", results=?sync_results);

        Ok(sync_results)
    }
}

pub async fn sync_venues() -> Result<i64, ErrorKind> {
    trace!("Syncing the venues");
    let client = reqwest::Client::new();
    let mut tivoli_syncer = TivoliSyncer::with_sender_and_client(Sender, &client);
    let result = tivoli_syncer.sync().await;
    match result {
        Err(err) => error!("Error syncing tivoli {}", err),
        _ => info!("All went well"),
    }
    Ok(3)
}
