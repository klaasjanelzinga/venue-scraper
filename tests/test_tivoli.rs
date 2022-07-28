mod common;

use async_trait::async_trait;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tracing::info;
use venue_scraper_api::errors::ErrorKind;
use venue_scraper_api::{HttpSend, TivoliSyncer};

pub struct MockSender {
    test_case: String,
    pub invoked_urls: Vec<String>,
}

#[async_trait]
impl HttpSend for MockSender {
    async fn send(
        &mut self,
        request: reqwest::RequestBuilder,
    ) -> Result<reqwest::Response, ErrorKind> {
        let request = request.build().unwrap();
        let url = request.url();

        self.invoked_urls.push(url.to_string());

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

        info!("Mocking url {} with {}", url, filename);
        let mut file = File::open(filename).await.unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await.unwrap();

        let response = http::response::Builder::new()
            .status(200)
            .body(buffer)
            .unwrap();

        Ok(response.into())
    }
}

#[tokio::test]
async fn test_sync_tivoli() {
    common::setup().await;

    let mock_sender = MockSender {
        test_case: String::from("default-test-case"),
        invoked_urls: Vec::new(),
    };

    let client = reqwest::Client::new();

    let mut tivoli_syncer = TivoliSyncer::with_sender_and_client(mock_sender, &client);
    let result = tivoli_syncer.sync().await;

    assert!(result.is_ok());
    let syncing_result = result.unwrap();
    assert_eq!(syncing_result.total_urls_fetched, 12);
    assert_eq!(syncing_result.total_items, 591);
    assert_eq!(syncing_result.total_unparseable_items, 0);

    assert_eq!(tivoli_syncer.http_sender.invoked_urls.len(), 12);

    ()
}
