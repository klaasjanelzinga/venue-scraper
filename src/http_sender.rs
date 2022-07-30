use crate::ErrorKind;
use async_trait::async_trait;
use reqwest::{RequestBuilder, Response};
use url::Url;

#[async_trait]
pub trait HttpSender {
    async fn send(&mut self, request: RequestBuilder) -> Result<Response, ErrorKind>;
}

pub struct DefaultHttpSender;

#[async_trait]
impl HttpSender for DefaultHttpSender {
    async fn send(&mut self, request: RequestBuilder) -> Result<Response, ErrorKind> {
        Ok(request.send().await?)
    }
}

/// Build a GET request object for execution by the client.
///
/// # Args:
/// - client: The request client.
/// - url: The url to use.
///
/// # Returns:
/// A get request or if the url cannot be parsed, an ErrorKind.
pub fn build_request_for_url(
    client: &reqwest::Client,
    url: &str,
) -> Result<RequestBuilder, ErrorKind> {
    let url = Url::parse(url)?;
    let request = client.get(url);
    Ok(request)
}

/// Get the result from the response if the response is a success. Otherwise, translate to an
/// ErrorKind.
///
/// # Args:
/// - response: The response to parse.
///
/// # Returns:
/// The body as a String or an ErrorKind if response was not a success.
pub async fn body_for_response(response: Response) -> Result<String, ErrorKind> {
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
