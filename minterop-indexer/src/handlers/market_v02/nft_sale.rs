use mb_sdk::events::mb_market_v02::*;

use crate::handlers::prelude::*;

pub(crate) async fn handle_nft_sold(
    rt: &TxProcessingRuntime,
    tx: &ReceiptData,
    data: serde_json::Value,
) {
    let data = match serde_json::from_value::<NftSaleData>(data.clone()) {
        Err(_) => {
            error!(r#"Invalid log for "nft_sold": {} ({:?})"#, data, tx);
            return;
        }
        Ok(data) => data,
    };

    future::join4(
        update_nft_listings(rt.clone(), tx.clone(), data.clone()),
        update_nft_offers(rt.clone(), tx.clone(), data.clone()),
        insert_nft_earnings(rt.clone(), tx.clone(), data.clone()),
        insert_nft_activities(rt.clone(), tx.clone(), data.clone()),
    )
    .await;
}

async fn update_nft_listings(
    rt: TxProcessingRuntime,
    tx: ReceiptData,
    data: NftSaleData,
) {
    use nft_listings::dsl;

    diesel::update(
        dsl::nft_listings
            .filter(dsl::nft_contract_id.eq(data.nft_contract_id.to_string()))
            .filter(dsl::token_id.eq(data.nft_token_id.to_string()))
            .filter(dsl::market_id.eq(tx.receiver.to_string()))
            .filter(dsl::approval_id.eq(pg_numeric(data.nft_approval_id))),
    )
    .set((
        dsl::accepted_at.eq(tx.timestamp),
        dsl::accepted_offer_id.eq(data.accepted_offer_id as i64),
    ))
    .execute_db(&rt.pg_connection, &tx, "update listing on sale")
    .await
}

async fn update_nft_offers(
    rt: TxProcessingRuntime,
    tx: ReceiptData,
    data: NftSaleData,
) {
    use nft_offers::dsl;

    diesel::update(
        dsl::nft_offers
            .filter(dsl::nft_contract_id.eq(data.nft_contract_id.to_string()))
            .filter(dsl::token_id.eq(data.nft_token_id.to_string()))
            .filter(dsl::market_id.eq(tx.receiver.to_string()))
            .filter(dsl::approval_id.eq(pg_numeric(data.nft_approval_id)))
            .filter(dsl::offer_id.eq(data.accepted_offer_id as i64)),
    )
    .set((dsl::accepted_at.eq(tx.timestamp),))
    .execute_db(&rt.pg_connection, &tx, "update listing on sale")
    .await
}

async fn insert_nft_earnings(
    rt: TxProcessingRuntime,
    tx: ReceiptData,
    mut data: NftSaleData,
) {
    let mut values = data
        .payout
        .drain()
        .map(|(receiver_id, amount)| NftEarning {
            token_id: data.nft_token_id.clone(),
            nft_contract_id: data.nft_contract_id.to_string(),
            market_id: tx.receiver.to_string(),
            approval_id: pg_numeric(data.nft_approval_id),
            offer_id: data.accepted_offer_id as i64,
            receipt_id: tx.id.clone(),
            timestamp: tx.timestamp,
            currency: data.currency.clone(),
            receiver_id: receiver_id.to_string(),
            amount: pg_numeric(amount.0),
            is_referral: false,
        })
        .collect::<Vec<_>>();

    if let Some(referrer_id) = data.referrer_id {
        values.push(NftEarning {
            token_id: data.nft_token_id.clone(),
            nft_contract_id: data.nft_contract_id.to_string(),
            market_id: tx.receiver.to_string(),
            approval_id: pg_numeric(data.nft_approval_id),
            offer_id: data.accepted_offer_id as i64,
            receipt_id: tx.id.clone(),
            timestamp: tx.timestamp,
            currency: data.currency.clone(),
            receiver_id: referrer_id.into(),
            amount: pg_numeric(data.referral_amount.unwrap().0),
            is_referral: true,
        });
    }

    diesel::insert_into(nft_earnings::table)
        .values(values)
        .execute_db(&rt.pg_connection, &tx, "insert earnings on sale")
        .await;
}

async fn insert_nft_activities(
    rt: TxProcessingRuntime,
    tx: ReceiptData,
    data: NftSaleData,
) {
    let activity = NftActivity {
        receipt_id: tx.id.clone(),
        tx_sender: tx.sender.to_string(),
        sender_pk: tx.sender_pk.clone(),
        timestamp: tx.timestamp,
        nft_contract_id: data.nft_contract_id.to_string(),
        token_id: data.nft_token_id.to_string(),
        kind: NFT_ACTIVITY_KIND_SOLD.to_string(),
        action_sender: None,
        action_receiver: None,
        memo: None,
        price: Some(pg_numeric(data.price.0)),
    };

    diesel::insert_into(nft_activities::table)
        .values(activity)
        .execute_db(&rt.pg_connection, &tx, "insert activity on sale")
        .await
}
