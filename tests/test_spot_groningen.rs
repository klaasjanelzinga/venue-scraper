use mock_sender::MockSender;
use venue_scraper_api::VenueScraper;

mod common;
mod mock_sender;

#[tokio::test]
async fn test_sync_spot_groningen() {
    common::setup().await;

    let mock_sender = MockSender {
        test_case: String::from("default-test-case"),
        invoked_urls: Vec::new(),
    };

    let client = reqwest::Client::new();

    let mut spot_groningen_syncer =
        VenueScraper::spot_groningen_with_sender_and_client(mock_sender, &client).unwrap();
    let result = spot_groningen_syncer.sync().await;

    assert!(result.is_ok());
    let syncing_result = result.unwrap();
    assert_eq!(syncing_result.total_urls_fetched, 1);
    assert_eq!(syncing_result.total_items, 439);
    assert_eq!(syncing_result.total_unparseable_items, 0);

    assert_eq!(spot_groningen_syncer.http_sender.invoked_urls.len(), 1);

    ()
}
