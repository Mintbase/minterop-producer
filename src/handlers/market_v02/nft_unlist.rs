use mb_sdk::events::mb_market_v02::*;

use crate::handlers::prelude::*;

pub(crate) async fn handle_nft_unlist(
    rt: &TxProcessingRuntime,
    tx: &ReceiptData,
    data: serde_json::Value,
) {
    let data = match serde_json::from_value::<NftUnlistData>(data.clone()) {
        Err(_) => {
            error!(r#"Invalid log for "nft_list": {} ({:?})"#, data, tx);
            return;
        }
        Ok(data) => data,
    };

    future::join(
        update_nft_listings(rt.clone(), tx.clone(), data.clone()),
        insert_nft_activities(rt.clone(), tx.clone(), data.clone()),
    )
    .await;
}

async fn update_nft_listings(
    rt: TxProcessingRuntime,
    tx: ReceiptData,
    data: NftUnlistData,
) {
    use nft_listings::dsl;

    diesel::update(
        dsl::nft_listings
            .filter(dsl::token_id.eq(data.nft_token_id.to_string()))
            .filter(dsl::nft_contract_id.eq(data.nft_contract_id.to_string()))
            .filter(dsl::market_id.eq(tx.receiver.to_string()))
            .filter(dsl::approval_id.eq(pg_numeric(data.nft_approval_id))),
    )
    .set(dsl::unlisted_at.eq(tx.timestamp))
    .execute_db(&rt.pg_connection, &tx, "update listing on unlist")
    .await
}

async fn insert_nft_activities(
    rt: TxProcessingRuntime,
    tx: ReceiptData,
    data: NftUnlistData,
) {
    let activity = NftActivity {
        receipt_id: tx.id.clone(),
        tx_sender: tx.sender.to_string(),
        sender_pk: tx.sender_pk.clone(),
        timestamp: tx.timestamp,
        nft_contract_id: data.nft_contract_id.to_string(),
        token_id: data.nft_token_id.to_string(),
        kind: NFT_ACTIVITY_KIND_UNLIST.to_string(),
        action_sender: tx.sender.to_string(),
        action_receiver: Some(tx.receiver.to_string()),
        memo: None,
        price: None,
        currency: None,
    };

    diesel::insert_into(nft_activities::table)
        .values(activity)
        .execute_db(&rt.pg_connection, &tx, "insert activity on unlist")
        .await
}
