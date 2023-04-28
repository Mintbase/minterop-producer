use mb_sdk::events::mb_market_v01::*;

use crate::handlers::prelude::*;

pub(crate) async fn handle_nft_unlist(
    rt: &TxProcessingRuntime,
    tx: &ReceiptData,
    data: serde_json::Value,
) {
    match serde_json::from_value::<Vec<NftUnlistLog>>(data.clone()) {
        Err(_) => error!(r#"Invalid log for "nft_list": {} ({:?})"#, data, tx),
        Ok(data_logs) => {
            future::join_all(
                data_logs.into_iter().map(|log| {
                    handle_nft_unlist_log(rt.clone(), tx.clone(), log)
                }),
            )
            .await;
        }
    }
}

async fn handle_nft_unlist_log(
    rt: TxProcessingRuntime,
    tx: ReceiptData,
    log: NftUnlistLog,
) {
    future::join3(
        update_nft_listings(rt.clone(), tx.clone(), log.clone()),
        invalidate_nft_offers(rt.clone(), tx.clone(), log.clone()),
        insert_nft_activities(rt.clone(), tx.clone(), log.clone()),
    )
    .await;
}

async fn update_nft_listings(
    rt: TxProcessingRuntime,
    tx: ReceiptData,
    log: NftUnlistLog,
) {
    use nft_listings::dsl;

    let (nft_contract, token_id, approval_id) =
        match super::parse_list_id(&log.list_id) {
            None => {
                crate::error!(
                    "Unparseable list ID: {}, ({:?})",
                    log.list_id,
                    tx
                );
                return;
            }
            Some(triple) => triple,
        };

    diesel::update(
        dsl::nft_listings
            .filter(dsl::token_id.eq(token_id.to_string()))
            .filter(dsl::nft_contract_id.eq(nft_contract.to_string()))
            .filter(dsl::approval_id.eq(pg_numeric(approval_id))),
    )
    .set(dsl::unlisted_at.eq(tx.timestamp))
    .execute_db(&rt.pg_connection, &tx, "update listing on unlist")
    .await
}

async fn invalidate_nft_offers(
    rt: TxProcessingRuntime,
    tx: ReceiptData,
    log: NftUnlistLog,
) {
    use minterop_data::schema::nft_offers::dsl;
    let (nft_contract, token_id, approval_id) =
        match super::parse_list_id(&log.list_id) {
            None => {
                crate::error!(
                    "Unparseable list ID: {}, ({:?})",
                    log.list_id,
                    tx
                );
                return;
            }
            Some(triple) => triple,
        };

    diesel::update(
        nft_offers::table
            .filter(dsl::nft_contract_id.eq(nft_contract.to_string()))
            .filter(dsl::token_id.eq(token_id.to_string()))
            .filter(dsl::market_id.eq(tx.receiver.to_string()))
            .filter(dsl::approval_id.eq(pg_numeric(approval_id)))
            .filter(dsl::accepted_at.is_null())
            .filter(dsl::withdrawn_at.is_null())
            .filter(dsl::outbid_at.is_null())
            .filter(dsl::invalidated_at.is_null()),
    )
    .set(dsl::invalidated_at.eq(tx.timestamp))
    .execute_db(&rt.pg_connection, &tx, "invalidate_offer")
    .await
}

async fn insert_nft_activities(
    rt: TxProcessingRuntime,
    tx: ReceiptData,
    log: NftUnlistLog,
) {
    let (nft_contract, token_id, _) = match super::parse_list_id(&log.list_id) {
        None => {
            crate::error!("Unparseable list ID: {}, ({:?})", log.list_id, tx);
            return;
        }
        Some(triple) => triple,
    };

    let activity = NftActivity {
        receipt_id: tx.id.clone(),
        tx_sender: tx.sender.to_string(),
        sender_pk: tx.sender_pk.clone(),
        timestamp: tx.timestamp,
        nft_contract_id: nft_contract.to_string(),
        token_id: token_id.to_string(),
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
