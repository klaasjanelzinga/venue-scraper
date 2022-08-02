use async_trait::async_trait;
use mongodb::Database;
use reqwest::ResponseBuilderExt;
use std::path::Path;
use std::rc::Rc;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tracing::trace;
use url::Url;
use venue_scraper_api::errors::ErrorKind;
use venue_scraper_api::http_sender::HttpSender;
use venue_scraper_api::VenueScraper;

pub struct MockSender {
    pub test_case: String,
}

#[async_trait]
impl HttpSender for MockSender {
    async fn send(&self, request: reqwest::RequestBuilder) -> Result<reqwest::Response, ErrorKind> {
        let request = request.build().unwrap();
        let url = request.url();

        let mut path = url.path().to_string();
        if path.starts_with("/") {
            path.remove(0);
        }
        if path.ends_with("/") {
            path.pop();
        }
        let host = url.host().unwrap();
        let mut filename = format!("tests/files/{}/{}/{}", host, self.test_case, path);
        let path = Path::new(filename.as_str());
        if path.is_dir() {
            filename = format!("{}/index", filename);
        }

        trace!("Mocking url {} with {}", url, filename);
        let file_result = File::open(filename).await;
        match file_result {
            Ok(mut file) => {
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer).await.unwrap();

                let response = http::response::Builder::new()
                    .url(url.clone())
                    .status(200)
                    .body(buffer)
                    .unwrap();

                Ok(response.into())
            }
            Err(_) => {
                let response = http::response::Builder::new()
                    .url(url.clone())
                    .status(404)
                    .body("")
                    .unwrap();
                Ok(response.into())
            }
        }
    }
}

#[allow(dead_code)]
pub fn spot_groningen_with_mock_sender(test_case: &str, db: Database) -> VenueScraper {
    let mock_sender = Rc::new(MockSender {
        test_case: String::from(test_case),
    });
    let client = reqwest::Client::new();
    VenueScraper::spot_groningen_with_sender_and_client(mock_sender, client, db).unwrap()
}

#[allow(dead_code)]
pub fn tivoli_utrecht_with_mock_sender(test_case: &str, db: Database) -> VenueScraper {
    let mock_sender = Rc::new(MockSender {
        test_case: String::from(test_case),
    });
    let client = reqwest::Client::new();
    VenueScraper::tivoli_with_sender_and_client(mock_sender, client, db).unwrap()
}
