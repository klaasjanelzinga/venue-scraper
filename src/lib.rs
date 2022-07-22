pub mod errors;

use std::fmt::{Debug, Display, Error, Formatter};
use async_trait::async_trait;
use errors::ErrorKind;
use log::{error, info, trace};
use reqwest::{RequestBuilder, Response, Url};
use scraper::{ElementRef, Html, Selector};
use scraper::html::Select;

#[derive(Debug)]
pub struct Agenda {
    title: String,
    description: String,
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

async fn request_for_url(client: &reqwest::Client, url: &str) -> Result<RequestBuilder, ErrorKind> {
    trace!("fetch_body_for_url({})", url);
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
    let  text = text_element.text().next().unwrap();
    Ok(text.trim().to_string())
}

fn get_select_on_element<'a>(search_in_element: &ElementRef<'a>, selector: &Selector) -> Result<ElementRef<'a>, ErrorKind> {
    let selected = search_in_element.select(&selector);
    if selected.count() != 1 {
        return Err(ErrorKind::CannotFindSelector {selector: format!("{:#?}", selector)});
    }
    Ok(search_in_element.select(selector).next().unwrap())
}

fn get_text_from_element(search_in: &ElementRef, selector: &Selector) -> Result<String, ErrorKind> {
    let selected = get_select_on_element(&search_in, &selector)?;
    get_text_for_single(&selected)
}

fn get_text_from_attr(search_in: &ElementRef, selector: &Selector, attr_name: &str) -> Result<String, ErrorKind> {
    let selected_element = get_select_on_element(&search_in, &selector)?;
    let attr = selected_element.value().attr(&attr_name);
    match attr {
        Some(value) => Ok(value.to_string()),
        None => Err(ErrorKind::CannotFindAttribute {attribute_name: attr_name.to_string()}),
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

    pub async fn sync(&mut self) -> Result<i64, ErrorKind> {

        let mut number_of_agenda_last_iteration = 0;
        let mut needs_next_page = true;

        for agenda_url in self.agenda_urls.iter() {
            if !needs_next_page {
                continue;
            }

            info!("Tivoli syncing. Url {}", agenda_url);
            let request = request_for_url(&self.client, &agenda_url).await?;
            let response = self.http_sender.send(request).await?;
            let body = body_for_response(response).await?;

            info!("Start parsing the response body");
            let parsed_html = Html::parse_document(&body);
            let selector = selector_for(r#"li.agenda-list-item"#)?;
            let title_selector = selector_for(r#"a.agenda-list-item__title-link"#)?;
            let description_selector = selector_for(r#"p.agenda-list-item__text"#)?;
            let url_selector = selector_for(r#"a.agenda-list-item__title-link"#)?;

            for element in parsed_html.select(&selector) {
                let title = get_text_from_element(&element, &title_selector)?;
                let description = get_text_from_element(&element, &description_selector)?;
                let url = get_text_from_attr(&element, &url_selector, "href")?;

                let agenda = Agenda {
                    title,
                    description,
                    url: url.to_string(),
                };
            }

            let number_of_agenda = parsed_html.select(&selector).count();
            needs_next_page = number_of_agenda >= number_of_agenda_last_iteration;
            number_of_agenda_last_iteration = number_of_agenda;
            info!("Number of agenda {}, last iteration {}", number_of_agenda, number_of_agenda_last_iteration);
        }
        Ok(13)
    }
}

pub async fn sync_venues() -> Result<i64, ErrorKind> {
    trace!("Syncing the venues");
    let client = reqwest::Client::new();
    let mut tivoli_syncer = TivoliSyncer::with_sender_and_client(Sender, &client);
    let result = tivoli_syncer.sync().await;
    match result {
        Err(err) => error!("Error syncing tivoli {}", err),
        _ => info!("All went well")
    }
    Ok(3)
}
