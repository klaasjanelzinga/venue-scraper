// src/lib.rs
#[cfg_attr(test, macro_use)] extern crate serde_json;

use std::error::Error;

use serde_json::Value;
use url::Url;

pub struct FooClient<S: HttpSend=Sender> {
    client: reqwest::Client,
    sender: S,
}

impl FooClient<Sender> {
    pub fn new() -> FooClient<Sender> {
        FooClient {
            client: reqwest::Client::new(),
            sender: Sender,
        }
    }
}

impl<S: HttpSend> FooClient<S> {

    pub fn with_sender(sender: S) -> FooClient<S> {
        FooClient {
            client: reqwest::Client::new(),
            sender: sender,
        }
    }

    pub fn get_widget(&self, id: &str)
         -> Result<Value, Box<Error>>
    {
        let url = Url::parse("https://example.com/widget/")?
            .join(id)?;
        let value: Value = self.sender
            .send(self.client.get(url))?
            .json()?;
        Ok(value)
    }
}

pub trait HttpSend {
    fn send(&self, request: reqwest::RequestBuilder)
        -> Result<reqwest::Response, Box<Error>>;
}

pub struct Sender;
impl HttpSend for Sender {
    fn send(&self, request: reqwest::RequestBuilder)
         -> Result<reqwest::Response, Box<Error>>
    {
        Ok(request.send()?)
    }
}

#[cfg(test)]
mod tests {
    use super::{FooClient, HttpSend};
    use std::error::Error;
    use std::cell::RefCell;
    use http::response;

    pub struct MockSender(
        RefCell<response::Builder>,
        &'static str
    );
    impl HttpSend for MockSender {
        fn send(&self, _: reqwest::RequestBuilder)
            -> Result<reqwest::Response, Box<Error>>
        {
            let mut builder = self.0.borrow_mut();
            let response = builder.body(self.1)?;
            let response = response.into();
            Ok(response)
        }
    }

    fn client_with_response(status: u16, body: &'static str)
         -> FooClient<MockSender>
    {
        let mut builder = response::Builder::new();
        builder.status(status);
        let sender = MockSender(RefCell::new(builder), body);
        FooClient::with_sender(sender)
    }

    #[test]
    fn get_widget() {
        let id = "42";
        let client = client_with_response(200, r#"{
              "id":42,
              "foo":"bar",
              "baz":"quux"
            }"#
        );
        let result = client.get_widget(id).expect("Call failed");
        assert_eq!(
                result,
                json!({
                    "id": 42,
                    "foo": "bar",
                    "baz": "quux"
                })
        );
    }
}