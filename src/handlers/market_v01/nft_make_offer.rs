use mb_sdk::events::mb_market_v01::*;

use crate::handlers::prelude::*;

pub(crate) async fn handle_nft_make_offer(
    rt: &TxProcessingRuntime,
    tx: &ReceiptData,
    data: serde_json::Value,
) {
    match serde_json::from_value::<Vec<NftMakeOfferLog>>(data.clone()) {
        Err(_) => {
            error!(r#"Invalid log for "nft_make_offer": {} ({:?})"#, data, tx);
        }
        Ok(data_logs) => {
            future::join_all(data_logs.into_iter().map(|log| {
                handle_nft_make_offer_log(rt.clone(), tx.clone(), log)
            }))
            .await;
        }
    }
}

async fn handle_nft_make_offer_log(
    rt: TxProcessingRuntime,
    tx: ReceiptData,
    log: NftMakeOfferLog,
) {
    future::join(
        insert_nft_offer(rt.clone(), tx.clone(), log.clone()),
        insert_nft_activities(rt.clone(), tx.clone(), log.clone()),
    )
    .await;
}

async fn insert_nft_offer(
    rt: TxProcessingRuntime,
    tx: ReceiptData,
    log: NftMakeOfferLog,
) {
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

    let offer = NftOffer {
        nft_contract_id: nft_contract.to_string(),
        token_id: token_id.to_string(),
        market_id: tx.receiver.to_string(),
        approval_id: pg_numeric(approval_id),
        currency: "near".to_string(),
        offer_price: pg_numeric(log.offer.price),
        offered_by: tx.sender.to_string(),
        offered_at: tx.timestamp,
        receipt_id: tx.id.clone(),
        offer_id: log.offer_num as i64,
        referrer_id: None,
        referral_amount: None,
        withdrawn_at: None,
        accepted_at: None,
        expires_at: Some(crate::nsecs_to_timestamp(log.offer.timeout)),
    };

    diesel::insert_into(nft_offers::table)
        .values(offer)
        .execute_db(&rt.pg_connection, &tx, "insert listing")
        .await
}

async fn insert_nft_activities(
    rt: TxProcessingRuntime,
    tx: ReceiptData,
    log: NftMakeOfferLog,
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
        kind: NFT_ACTIVITY_KIND_MAKE_OFFER.to_string(),
        action_sender: None,
        action_receiver: None,
        memo: None,
        price: Some(pg_numeric(log.offer.price)),
    };

    diesel::insert_into(nft_activities::table)
        .values(activity)
        .execute_db(&rt.pg_connection, &tx, "insert activity on make offer")
        .await
}
