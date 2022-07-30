use async_trait::async_trait;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tracing::trace;
use venue_scraper_api::errors::ErrorKind;
use venue_scraper_api::http_sender::HttpSender;

pub struct MockSender {
    pub test_case: String,
}

#[async_trait]
impl HttpSender for MockSender {
    async fn send(
        &mut self,
        request: reqwest::RequestBuilder,
    ) -> Result<reqwest::Response, ErrorKind> {
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
