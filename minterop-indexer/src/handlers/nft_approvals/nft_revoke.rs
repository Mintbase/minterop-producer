use mb_sdk::events::nft_approvals::NftRevokeData;

use crate::handlers::prelude::*;

pub(crate) async fn handle_nft_revoke(
    rt: &TxProcessingRuntime,
    tx: &ReceiptData,
    data: serde_json::Value,
) {
    let data = match serde_json::from_value::<NftRevokeData>(data.clone()) {
        Err(_) => {
            error!(r#"Invalid log for "nft_transfer": {} ({:?})"#, data, tx);
            return;
        }
        Ok(data) => data,
    };

    future::join(
        delete_nft_approvals(rt.clone(), tx.clone(), data.clone()),
        insert_nft_activities(rt.clone(), tx.clone(), data.clone()),
    )
    .await;
}

async fn delete_nft_approvals(
    rt: TxProcessingRuntime,
    tx: ReceiptData,
    data: NftRevokeData,
) {
    use minterop_common::schema::nft_approvals::dsl;

    diesel::delete(
        nft_approvals::table
            .filter(dsl::nft_contract_id.eq(tx.receiver.to_string()))
            .filter(dsl::token_id.eq(data.token_id))
            .filter(dsl::approved_account_id.eq(data.account_id)),
    )
    .execute_db(&rt.pg_connection, &tx, "delete approval on revoke")
    .await;
}

async fn insert_nft_activities(
    rt: TxProcessingRuntime,
    tx: ReceiptData,
    log: NftRevokeData,
) {
    diesel::insert_into(nft_activities::table)
        .values(NftActivity {
            receipt_id: tx.id.clone(),
            tx_sender: tx.sender.to_string(),
            sender_pk: tx.sender_pk.clone(),
            timestamp: tx.timestamp,
            nft_contract_id: tx.receiver.to_string(),
            token_id: log.token_id,
            kind: NFT_ACTIVITY_KIND_REVOKE.to_string(),
            action_sender: Some(tx.sender.to_string()),
            action_receiver: Some(log.account_id.to_string()),
            memo: None,
            price: None,
        })
        .execute_db(&rt.pg_connection, &tx, "insert activity on transfer")
        .await
}
