use mock_sender::spot_groningen_with_mock_sender;

mod common;
mod mock_sender;

#[tokio::test]
async fn test_sync_spot_groningen() {
    let test_fixtures = common::setup().await;

    let spot_groningen_syncer =
        spot_groningen_with_mock_sender("default-test-case", test_fixtures.db.clone());
    let result = spot_groningen_syncer.sync().await;

    assert!(result.is_ok());
    let syncing_result = result.unwrap();
    assert_eq!(syncing_result.total_urls_fetched, 1);
    assert_eq!(syncing_result.total_items, 439);
    assert_eq!(syncing_result.total_unparseable_items, 0);

    ()
}
