use mock_sender::MockSender;
use std::rc::Rc;
use venue_scraper_api::sync_venues;

mod common;
mod mock_sender;

/// Sync all venues, each venue must have a 'default-test-case'
#[tokio::test]
async fn test_sync_all_venues() {
    let test_fixtures = common::setup().await;

    let mock_sender = Rc::new(MockSender {
        test_case: String::from("default-test-case"),
    });

    let client = reqwest::Client::new();
    let result = sync_venues(&client, &test_fixtures.db, mock_sender).await;
    assert!(result.is_ok());

    let syncing_results = result.unwrap();
    assert_eq!(syncing_results.total_items, 1030);
    assert_eq!(syncing_results.total_items_inserted, 1030);
    assert_eq!(syncing_results.total_unparseable_items, 0);
    assert_eq!(syncing_results.total_urls_fetched, 13);

    ()
}
