mod common;
mod mock_sender;

use mock_sender::tivoli_utrecht_with_mock_sender;

#[tokio::test]
async fn test_sync_tivoli() {
    let test_fixtures = common::setup().await;

    let tivoli_syncer =
        tivoli_utrecht_with_mock_sender("default-test-case", test_fixtures.db.clone());
    let result = tivoli_syncer.sync().await;

    assert!(result.is_ok());
    let syncing_result = result.unwrap();
    assert_eq!(syncing_result.total_urls_fetched, 12);
    assert_eq!(syncing_result.total_items, 591);
    assert_eq!(syncing_result.total_items_inserted, 591);
    assert_eq!(syncing_result.total_unparseable_items, 0);

    ()
}
