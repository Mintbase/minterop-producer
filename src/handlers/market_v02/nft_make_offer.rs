use mb_sdk::events::mb_market_v02::*;

use crate::handlers::prelude::*;

pub(crate) async fn handle_nft_make_offer(
    rt: &TxProcessingRuntime,
    tx: &ReceiptData,
    data: serde_json::Value,
) {
    let data = match serde_json::from_value::<NftMakeOfferData>(data.clone()) {
        Err(_) => {
            error!(r#"Invalid log for "nft_make_offer": {} ({:?})"#, data, tx);
            return;
        }
        Ok(data) => data,
    };

    future::join(
        insert_nft_offer(rt.clone(), tx.clone(), data.clone()),
        insert_nft_activities(rt.clone(), tx.clone(), data.clone()),
    )
    .await;
}

async fn insert_nft_offer(
    rt: TxProcessingRuntime,
    tx: ReceiptData,
    data: NftMakeOfferData,
) {
    let offer = NftOffer {
        nft_contract_id: data.nft_contract_id.to_string(),
        token_id: data.nft_token_id.to_string(),
        market_id: tx.receiver.to_string(),
        approval_id: pg_numeric(data.nft_approval_id),
        currency: data.currency,
        offer_price: pg_numeric(data.price.0),
        offered_by: data.offerer_id.to_string(),
        offered_at: tx.timestamp,
        receipt_id: tx.id.clone(),
        offer_id: data.offer_id as i64,
        referrer_id: data
            .affiliate_id
            .as_ref()
            .map(|account| account.to_string()),
        referral_amount: data
            .affiliate_amount
            .map(|balance| pg_numeric(balance.0)),
        affiliate_id: data.affiliate_id.map(|account| account.to_string()),
        affiliate_amount: data
            .affiliate_amount
            .map(|balance| pg_numeric(balance.0)),
        withdrawn_at: None,
        accepted_at: None,
        invalidated_at: None,
        outbid_at: None,
        expires_at: None,
    };

    diesel::insert_into(nft_offers::table)
        .values(offer)
        .execute_db(&rt.pg_connection, &tx, "insert listing")
        .await
}

async fn insert_nft_activities(
    rt: TxProcessingRuntime,
    tx: ReceiptData,
    data: NftMakeOfferData,
) {
    if let Some((lister, currency)) = crate::database::query_lister_currency(
        data.nft_contract_id.to_string(),
        data.nft_token_id.clone(),
        tx.receiver.to_string(),
        data.nft_approval_id,
        &rt.pg_connection,
    )
    .await
    {
        let activity = NftActivity {
            receipt_id: tx.id.clone(),
            tx_sender: tx.sender.to_string(),
            sender_pk: tx.sender_pk.clone(),
            timestamp: tx.timestamp,
            nft_contract_id: data.nft_contract_id.to_string(),
            token_id: data.nft_token_id,
            kind: NFT_ACTIVITY_KIND_MAKE_OFFER.to_string(),
            action_sender: tx.sender.to_string(),
            action_receiver: Some(lister),
            memo: None,
            price: Some(pg_numeric(data.price.0)),
            currency: Some(currency),
        };

        diesel::insert_into(nft_activities::table)
            .values(activity)
            .execute_db(&rt.pg_connection, &tx, "insert activity on make offer")
            .await
    }
}
