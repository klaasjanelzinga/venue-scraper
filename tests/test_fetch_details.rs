mod common;
mod mock_sender;

use mock_sender::spot_groningen_with_mock_sender;

/// Test if only new items in need of details are fetched.
/// 1. Make a regular sync.
/// 2. Fetch the details for fetched agenda items.
/// 3. Make the next run of the regular sync.
/// 4. Only details fetched in step 3 should be downloaded.
#[tokio::test]
async fn test_sync_with_details() {
    let test_fixtures = common::setup().await;

    // 1. Fetch the program. This should give 6 agenda items.
    let spot_groningen_syncer =
        spot_groningen_with_mock_sender("details-test-case", test_fixtures.db.clone());
    let result = spot_groningen_syncer.sync().await;

    assert!(result.is_ok());
    let syncing_result = result.unwrap();
    assert_eq!(syncing_result.total_urls_fetched, 1);
    assert_eq!(syncing_result.total_items, 6);
    assert_eq!(syncing_result.total_items_inserted, 6);

    // 2. Next, sync the details.
    let details_result = spot_groningen_syncer.sync_details().await;
    assert!(details_result.is_ok());
    let syncing_result = details_result.unwrap();
    assert_eq!(syncing_result.total_urls_fetched, 6);
    assert_eq!(syncing_result.total_items_updated, 6);
    assert_eq!(syncing_result.total_items_inserted, 0);

    // 3. The next run. Fetch the program.
    let spot_groningen_syncer =
        spot_groningen_with_mock_sender("details-test-case-run-2", test_fixtures.db.clone());
    let result = spot_groningen_syncer.sync().await;
    assert!(result.is_ok());
    let syncing_result = result.unwrap();
    assert_eq!(syncing_result.total_urls_fetched, 1);
    assert_eq!(syncing_result.total_items, 10);
    assert_eq!(syncing_result.total_items_inserted, 4);

    // 4. Fetch the details. Only the items in need of details should fetch.
    let result = spot_groningen_syncer.sync_details().await;
    assert!(result.is_ok());
    let syncing_result = result.unwrap();
    assert_eq!(syncing_result.total_urls_fetched, 4);
    assert_eq!(syncing_result.total_items_updated, 4);
    assert_eq!(syncing_result.total_items_inserted, 0);

    ()
}
