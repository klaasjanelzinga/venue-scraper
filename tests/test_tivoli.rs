mod common;
mod mock_sender;

use mock_sender::MockSender;
use venue_scraper_api::VenueScraper;

#[tokio::test]
async fn test_sync_tivoli() {
    let test_fixtures = common::setup().await;

    let mock_sender = Box::new(MockSender {
        test_case: String::from("default-test-case"),
    });

    let client = reqwest::Client::new();

    let mut tivoli_syncer =
        VenueScraper::tivoli_with_sender_and_client(mock_sender, client, test_fixtures.db)
            .unwrap();
    let result = tivoli_syncer.sync().await;

    assert!(result.is_ok());
    let syncing_result = result.unwrap();
    assert_eq!(syncing_result.total_urls_fetched, 12);
    assert_eq!(syncing_result.total_items, 591);
    assert_eq!(syncing_result.total_unparseable_items, 0);

    ()
}
