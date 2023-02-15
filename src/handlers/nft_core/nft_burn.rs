use mb_sdk::events::nft_core::NftBurnLog;

use crate::{
    error,
    handlers::prelude::*,
    runtime::TxProcessingRuntime,
    ReceiptData,
};

pub(crate) async fn handle_nft_burn(
    rt: &TxProcessingRuntime,
    tx: &ReceiptData,
    data: serde_json::Value,
) {
    // contract should always be inserted prior to token for metadata resolve
    rt.minterop_rpc.contract(tx.receiver.clone(), false).await;

    match serde_json::from_value::<Vec<NftBurnLog>>(data.clone()) {
        Err(_) => error!(r#"Invalid log for "nft_burn": {} ({:?})"#, data, tx),
        Ok(data_logs) => {
            future::join_all(
                data_logs
                    .into_iter()
                    .map(|log| handle_nft_burn_log(rt, tx, log)),
            )
            .await;
        }
    }
}

async fn handle_nft_burn_log(
    rt: &TxProcessingRuntime,
    tx: &ReceiptData,
    log: NftBurnLog,
) {
    future::join4(
        insert_nft_tokens(rt.clone(), tx.clone(), log.clone()),
        insert_nft_activities(rt.clone(), tx.clone(), log.clone()),
        crate::handlers::invalidate_nft_listings(
            rt.clone(),
            tx.clone(),
            tx.receiver.to_string(),
            log.token_ids.clone(),
            None,
        ),
        crate::handlers::invalidate_nft_offers(
            rt.clone(),
            tx.clone(),
            tx.receiver.to_string(),
            log.token_ids.clone(),
            None,
        ),
    )
    .await;
}

async fn insert_nft_tokens(
    rt: TxProcessingRuntime,
    tx: ReceiptData,
    log: NftBurnLog,
) {
    use minterop_data::schema::nft_tokens::dsl;

    let tokens = log
        .token_ids
        .iter()
        .map(|token_id| NftToken {
            token_id: token_id.clone(),
            nft_contract_id: tx.receiver.to_string(),
            owner: tx.sender.to_string(),
            burned_timestamp: Some(tx.timestamp),
            burned_receipt_id: Some(tx.id.clone()),
            ..NftToken::empty()
        })
        .collect::<Vec<_>>();

    diesel::insert_into(nft_tokens::table)
        .values(tokens)
        .on_conflict(diesel::pg::upsert::on_constraint("nft_tokens_pkey"))
        .do_update()
        .set((
            dsl::burned_timestamp.eq(tx.timestamp),
            dsl::burned_receipt_id.eq(tx.id.clone()),
        ))
        .execute_db(&rt.pg_connection, &tx, "insert token on transfer")
        .await
}

async fn insert_nft_activities(
    rt: TxProcessingRuntime,
    tx: ReceiptData,
    log: NftBurnLog,
) {
    let activities = log
        .token_ids
        .iter()
        .map(|token_id| NftActivity {
            receipt_id: tx.id.clone(),
            tx_sender: tx.sender.to_string(),
            sender_pk: tx.sender_pk.clone(),
            timestamp: tx.timestamp,
            nft_contract_id: tx.receiver.to_string(),
            token_id: token_id.clone(),
            kind: NFT_ACTIVITY_KIND_BURN.to_string(),
            action_sender: tx.sender.to_string(),
            action_receiver: None,
            memo: None,
            price: None,
        })
        .collect::<Vec<_>>();

    diesel::insert_into(nft_activities::table)
        .values(activities)
        .execute_db(&rt.pg_connection, &tx, "insert activity on mint")
        .await
}
