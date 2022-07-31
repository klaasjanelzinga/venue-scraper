mod common;
mod mock_sender;

use mock_sender::MockSender;
use std::rc::Rc;
use venue_scraper_api::VenueScraper;

/// Run the sync operation twice, simulating multiple fetches.
///
/// The first run, 52 agenda items are inserted.
/// The second run, the same 52 items + 4 new ones are found. Only the new ones are inserted.
#[tokio::test]
async fn test_sync_multi_sync() {
    let test_fixtures = common::setup().await;
    let mock_sender = Rc::new(MockSender {
        test_case: String::from("multiple-fetch-run-1"),
    });
    let client = reqwest::Client::new();

    let tivoli_syncer = VenueScraper::tivoli_with_sender_and_client(
        mock_sender,
        client.clone(),
        test_fixtures.db.clone(),
    )
    .unwrap();
    let result = tivoli_syncer.sync().await;

    assert!(result.is_ok());
    let syncing_result = result.unwrap();
    assert_eq!(syncing_result.total_items, 52);
    assert_eq!(syncing_result.total_items_inserted, 52);

    let mock_sender = Rc::new(MockSender {
        test_case: String::from("multiple-fetch-run-2"),
    });
    let tivoli_syncer = VenueScraper::tivoli_with_sender_and_client(
        mock_sender,
        client.clone(),
        test_fixtures.db.clone(),
    )
    .unwrap();
    let result = tivoli_syncer.sync().await;
    assert!(result.is_ok());
    let syncing_result = result.unwrap();
    assert_eq!(syncing_result.total_items, 56);
    assert_eq!(syncing_result.total_items_inserted, 4);

    ()
}
