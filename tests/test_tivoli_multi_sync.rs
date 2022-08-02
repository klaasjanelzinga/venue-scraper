mod common;
mod mock_sender;

use mock_sender::tivoli_utrecht_with_mock_sender;

/// Run the sync operation twice, simulating multiple fetches.
///
/// The first run, 52 agenda items are inserted.
/// The second run, the same 52 items + 4 new ones are found. Only the new ones are inserted.
#[tokio::test]
async fn test_sync_multi_sync() {
    let test_fixtures = common::setup().await;
    let tivoli_syncer =
        tivoli_utrecht_with_mock_sender("multiple-fetch-run-1", test_fixtures.db.clone());

    let result = tivoli_syncer.sync().await;

    assert!(result.is_ok());
    let syncing_result = result.unwrap();
    assert_eq!(syncing_result.total_items, 52);
    assert_eq!(syncing_result.total_items_inserted, 52);

    // Run 2.
    let tivoli_syncer =
        tivoli_utrecht_with_mock_sender("multiple-fetch-run-2", test_fixtures.db.clone());
    let result = tivoli_syncer.sync().await;
    assert!(result.is_ok());
    let syncing_result = result.unwrap();
    assert_eq!(syncing_result.total_items, 56);
    assert_eq!(syncing_result.total_items_inserted, 4);

    ()
}
