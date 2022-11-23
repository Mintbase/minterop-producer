use mb_sdk::events::mb_market_v02::*;

use crate::handlers::prelude::*;

pub(crate) async fn handle_nft_list(
    rt: &TxProcessingRuntime,
    tx: &ReceiptData,
    data: serde_json::Value,
) {
    // TODO: unknown token contract?

    let data = match serde_json::from_value::<NftListData>(data.clone()) {
        Err(_) => {
            error!(r#"Invalid log for "nft_list": {} ({:?})"#, data, tx);
            return;
        }
        Ok(data) => data,
    };

    future::join4(
        insert_nft_listing(rt.clone(), tx.clone(), data.clone()),
        insert_nft_activities(rt.clone(), tx.clone(), data.clone()),
        crate::handlers::invalidate_nft_listings(
            rt.clone(),
            tx.clone(),
            data.nft_contract_id.to_string(),
            vec![data.nft_token_id.clone()],
            Some(tx.receiver.to_string()),
        ),
        crate::handlers::invalidate_nft_offers(
            rt.clone(),
            tx.clone(),
            data.nft_contract_id.to_string(),
            vec![data.nft_token_id.clone()],
            Some(tx.receiver.to_string()),
        ),
    )
    .await;
}

async fn insert_nft_listing(
    rt: TxProcessingRuntime,
    tx: ReceiptData,
    data: NftListData,
) {
    let metadata_id = crate::database::query_metadata_id(
        data.nft_contract_id.to_string(),
        data.nft_token_id.clone(),
        &rt.pg_connection,
    )
    .await;
    if metadata_id.is_none() {
        crate::warn!("Failed to find metadata ID ({:?})", tx);
    }

    let listing = NftListing {
        nft_contract_id: data.nft_contract_id.to_string(),
        token_id: data.nft_token_id,
        market_id: tx.receiver.to_string(),
        approval_id: pg_numeric(data.nft_approval_id),
        created_at: tx.timestamp,
        receipt_id: tx.id.clone(),
        kind: data.kind,
        price: Some(pg_numeric(data.price.0)),
        currency: data.currency,
        listed_by: data.nft_owner_id.to_string(),
        unlisted_at: None,
        accepted_at: None,
        accepted_offer_id: None,
        invalidated_at: None,
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
    data: NftListData,
) {
    let activity = NftActivity {
        receipt_id: tx.id.clone(),
        tx_sender: tx.sender.to_string(),
        sender_pk: tx.sender_pk.clone(),
        timestamp: tx.timestamp,
        nft_contract_id: data.nft_contract_id.to_string(),
        token_id: data.nft_token_id,
        kind: NFT_ACTIVITY_KIND_LIST.to_string(),
        action_sender: None,
        action_receiver: None,
        memo: None,
        price: Some(pg_numeric(data.price.0)),
    };

    diesel::insert_into(nft_activities::table)
        .values(activity)
        .execute_db(&rt.pg_connection, &tx, "insert activity on listing")
        .await
}
