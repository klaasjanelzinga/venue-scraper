pub mod errors;

use async_trait::async_trait;
use log::{info, trace};
use reqwest::{Response, RequestBuilder, Url};
use scraper::{Html, Selector};
use errors::{ErrorKind};


pub struct Agenda {
    title: String,
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
        })
    }
    let body = response.text().await?;
    Ok(body)
}


impl<HttpSender: HttpSend> TivoliSyncer<'_, HttpSender> {

    pub fn with_sender_and_client(http_sender: HttpSender, client: &reqwest::Client) -> TivoliSyncer<HttpSender> {
        let mut agenda_urls = Vec::new();
        agenda_urls.push(String::from("https://www.tivolivredenburg.nl/agenda/"));
        for count in 2..20 {
            agenda_urls.push(format!("https://www.tivolivredenburg.nl/agenda/page/{}/", count));
        }

        TivoliSyncer {
            client,
            http_sender,
            agenda_urls,
        }
    }

    pub async fn sync(&mut self) -> Result<i64, ErrorKind> {

        info!("Count total agenda items on this page.");
        info!("Create a agenda informational");
        info!("Check if this information is new, updated");
        info!("If information is updated or new, update insert");
        info!("Is extra information needed? If so fetch.");
        info!("If has next, goto 1.");

        for agenda_url in self.agenda_urls.iter() {
            info!("Tivoli syncing. Url {}", agenda_url);
            let request = request_for_url(&self.client,&agenda_url).await?;
            let response = self.http_sender.send(request).await?;
            let body = body_for_response(response).await?;

            info!("Start parsing the response body");
            let parsed_html = Html::parse_document(&body);
            let selector = Selector::parse(r#"li.agenda-list-item"#).unwrap();
            let title_selector = Selector::parse(r#"a.agenda-list-item__title-link"#).unwrap();
            for element in parsed_html.select(&selector) {
                let title = element.select(&title_selector).next().unwrap();
                info!("{}", title.text().next().unwrap());

                let agenda = Agenda {
                    title: title.text().next().unwrap().to_string(),
                };
            }
        }
        Ok(13)
    }
}



pub async fn sync_venues() -> Result<i64, ErrorKind> {
    trace!("Syncing the venues");
    let client = reqwest::Client::new();
    let mut tivoli_syncer = TivoliSyncer::with_sender_and_client(Sender, &client);
    let result = tivoli_syncer.sync().await;
    if result.is_ok() {
        println!("{}", result.unwrap())
    } else {
        println!("error")
    }
    Ok(3)
}