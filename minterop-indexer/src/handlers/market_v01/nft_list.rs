use mb_sdk::events::mb_market_v01::*;

use crate::handlers::prelude::*;

pub(crate) async fn handle_nft_list(
    rt: &TxProcessingRuntime,
    tx: &ReceiptData,
    data: serde_json::Value,
) {
    // TODO: unknown token contract?

    match serde_json::from_value::<Vec<NftListLog>>(data.clone()) {
        Err(_) => error!(r#"Invalid log for "nft_list": {} ({:?})"#, data, tx),
        Ok(data_logs) => {
            future::join_all(
                data_logs.into_iter().map(|log| {
                    handle_nft_list_log(rt.clone(), tx.clone(), log)
                }),
            )
            .await;
        }
    }
}

async fn handle_nft_list_log(
    rt: TxProcessingRuntime,
    tx: ReceiptData,
    log: NftListLog,
) {
    future::join(
        insert_nft_listing(rt.clone(), tx.clone(), log.clone()),
        insert_nft_activities(rt.clone(), tx.clone(), log.clone()),
    )
    .await;
}

async fn insert_nft_listing(
    rt: TxProcessingRuntime,
    tx: ReceiptData,
    log: NftListLog,
) {
    let approval_id = log.approval_id.parse().unwrap();
    let kind = if log.autotransfer {
        NFT_LISTING_KIND_SIMPLE.to_string()
    } else {
        NFT_LISTING_KIND_AUCTION.to_string()
    };

    let metadata_id = match crate::database::query_metadata_id(
        log.store_id.clone(),
        log.token_id.clone(),
        &rt.pg_connection,
    )
    .await
    {
        None => {
            crate::error!("Failed to find metadata ID ({:?})", tx);
            return;
        }
        Some(metadata_id) => metadata_id,
    };

    let listing = NftListing {
        nft_contract_id: log.store_id,
        token_id: log.token_id,
        market_id: tx.receiver.to_string(),
        approval_id,
        created_at: tx.timestamp,
        receipt_id: tx.id.clone(),
        kind,
        price: Some(pg_numeric(log.price.0)),
        currency: "near".to_string(),
        listed_by: log.owner_id,
        unlisted_at: None,
        accepted_at: None,
        accepted_offer_id: None,
        metadata_id,
    };

    diesel::insert_into(nft_listings::table)
        .values(listing)
        .execute_db(&rt.pg_connection, &tx, "insert listing")
        .await
}

async fn insert_nft_activities(
    rt: TxProcessingRuntime,
    tx: ReceiptData,
    log: NftListLog,
) {
    let activity = NftActivity {
        receipt_id: tx.id.clone(),
        tx_sender: tx.sender.to_string(),
        sender_pk: tx.sender_pk.clone(),
        timestamp: tx.timestamp,
        nft_contract_id: log.store_id,
        token_id: log.token_id,
        kind: NFT_ACTIVITY_KIND_LIST.to_string(),
        action_sender: None,
        action_receiver: None,
        memo: None,
        price: Some(pg_numeric(log.price.0)),
    };

    diesel::insert_into(nft_activities::table)
        .values(activity)
        .execute_db(&rt.pg_connection, &tx, "insert activity on listing")
        .await
}
