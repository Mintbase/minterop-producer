use mb_sdk::events::mb_market_v01::*;

use crate::handlers::prelude::*;

pub(crate) async fn handle_nft_withdraw_offer(
    rt: &TxProcessingRuntime,
    tx: &ReceiptData,
    data: serde_json::Value,
) {
    let data =
        match serde_json::from_value::<NftWithdrawOfferData>(data.clone()) {
            Err(_) => {
                error!(
                    r#"Invalid log for "nft_widthdraw_offer": {} ({:?})"#,
                    data, tx
                );
                return;
            }
            Ok(data) => data,
        };

    future::join(
        update_nft_offer(rt.clone(), tx.clone(), data.clone()),
        insert_nft_activities(rt.clone(), tx.clone(), data.clone()),
    )
    .await;
}

async fn update_nft_offer(
    rt: TxProcessingRuntime,
    tx: ReceiptData,
    data: NftWithdrawOfferData,
) {
    use nft_offers::dsl;

    let (nft_contract, token_id, _) = match super::parse_list_id(&data.list_id)
    {
        None => {
            crate::error!("Unparseable list ID: {}, ({:?})", data.list_id, tx);
            return;
        }
        Some(triple) => triple,
    };

    diesel::update(
        dsl::nft_offers
            .filter(dsl::nft_contract_id.eq(nft_contract.to_string()))
            .filter(dsl::token_id.eq(token_id.to_string()))
            .filter(dsl::market_id.eq(tx.receiver.to_string()))
            .filter(dsl::offer_id.eq(data.offer_num as i64)),
    )
    .set(dsl::withdrawn_at.eq(tx.timestamp))
    .execute_db(&rt.pg_connection, &tx, "update offer on withdrawal")
    .await
}

async fn insert_nft_activities(
    rt: TxProcessingRuntime,
    tx: ReceiptData,
    data: NftWithdrawOfferData,
) {
    let (nft_contract, token_id, approval_id) =
        match super::parse_list_id(&data.list_id) {
            None => {
                crate::error!(
                    "Unparseable list ID: {}, ({:?})",
                    data.list_id,
                    tx
                );
                return;
            }
            Some(triple) => triple,
        };

    let lister = crate::database::query_lister(
        nft_contract.to_string(),
        token_id.to_string(),
        tx.receiver.to_string(),
        approval_id,
        &rt.pg_connection,
    )
    .await;

    let activity = NftActivity {
        receipt_id: tx.id.clone(),
        tx_sender: tx.sender.to_string(),
        sender_pk: tx.sender_pk.clone(),
        timestamp: tx.timestamp,
        nft_contract_id: nft_contract.to_string(),
        token_id: token_id.to_string(),
        kind: NFT_ACTIVITY_KIND_WITHDRAW_OFFER.to_string(),
        action_sender: tx.sender.to_string(),
        action_receiver: lister,
        memo: None,
        price: None,
    };

    diesel::insert_into(nft_activities::table)
        .values(activity)
        .execute_db(&rt.pg_connection, &tx, "insert activity on withdraw offer")
        .await
}
