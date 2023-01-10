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

    // cannot use future::join_all, because it doesn't allow me to generate the
    // array/vector without boxing futures
    future::join3(
        future::join3(
            update_nft_listings(rt.clone(), tx.clone(), data.clone()),
            update_nft_offers(rt.clone(), tx.clone(), data.clone()),
            insert_nft_earnings(rt.clone(), tx.clone(), data.clone()),
        ),
        future::join3(
            insert_nft_activities(rt.clone(), tx.clone(), data.clone()),
            remove_listing_invalidation(rt.clone(), tx.clone(), data.clone()),
            remove_offer_invalidation(rt.clone(), tx.clone(), data.clone()),
        ),
        dispatch_sale_event(rt.clone(), tx.clone(), data.clone()),
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
            is_mintbase_cut: false,
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
            is_mintbase_cut: false,
        });
    }

    if let Some(mb_earning) = data.mintbase_amount {
        values.push(NftEarning {
            token_id: data.nft_token_id.clone(),
            nft_contract_id: data.nft_contract_id.to_string(),
            market_id: tx.receiver.to_string(),
            approval_id: pg_numeric(data.nft_approval_id),
            offer_id: data.accepted_offer_id as i64,
            receipt_id: tx.id.clone(),
            timestamp: tx.timestamp,
            currency: data.currency.clone(),
            receiver_id: tx.receiver.to_string(),
            amount: pg_numeric(mb_earning.0),
            is_referral: false,
            is_mintbase_cut: true,
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
    if let (lister, Some(offerer)) = crate::database::query_lister_and_offerer(
        data.nft_contract_id.to_string(),
        data.nft_token_id.clone(),
        tx.receiver.to_string(),
        data.nft_approval_id,
        data.accepted_offer_id,
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
            token_id: data.nft_token_id.to_string(),
            kind: NFT_ACTIVITY_KIND_SOLD.to_string(),
            action_sender: offerer,
            action_receiver: lister,
            memo: None,
            price: Some(pg_numeric(data.price.0)),
        };

        diesel::insert_into(nft_activities::table)
            .values(activity)
            .execute_db(&rt.pg_connection, &tx, "insert activity on sale")
            .await
    }
}

async fn remove_listing_invalidation(
    rt: crate::runtime::TxProcessingRuntime,
    tx: crate::ReceiptData,
    data: NftSaleData,
) {
    use minterop_data::schema::nft_listings::dsl;

    use crate::handlers::prelude::*;

    diesel::update(
        nft_listings::table
            .filter(dsl::nft_contract_id.eq(data.nft_contract_id.to_string()))
            .filter(dsl::token_id.eq(data.nft_token_id))
            .filter(dsl::approval_id.eq(pg_numeric(data.nft_approval_id)))
            .filter(dsl::market_id.eq(tx.receiver.to_string())),
    )
    .set(dsl::invalidated_at.eq(Option::<chrono::NaiveDateTime>::None))
    .execute_db(&rt.pg_connection, &tx, "revalidate listing")
    .await
}

async fn remove_offer_invalidation(
    rt: crate::runtime::TxProcessingRuntime,
    tx: crate::ReceiptData,
    data: NftSaleData,
) {
    use minterop_data::schema::nft_offers::dsl;

    use crate::handlers::prelude::*;

    diesel::update(
        nft_offers::table
            .filter(dsl::nft_contract_id.eq(data.nft_contract_id.to_string()))
            .filter(dsl::token_id.eq(data.nft_token_id))
            .filter(dsl::approval_id.eq(pg_numeric(data.nft_approval_id)))
            .filter(dsl::market_id.eq(tx.receiver.to_string()))
            .filter(dsl::offer_id.eq(data.accepted_offer_id as i64)),
    )
    .set(dsl::invalidated_at.eq(Option::<chrono::NaiveDateTime>::None))
    .execute_db(&rt.pg_connection, &tx, "revalidate offer")
    .await
}

async fn dispatch_sale_event(
    rt: crate::runtime::TxProcessingRuntime,
    tx: crate::ReceiptData,
    data: NftSaleData,
) {
    if let Some(offerer) = crate::database::query_offerer(
        data.nft_contract_id.to_string(),
        data.nft_token_id.clone(),
        tx.receiver.to_string(),
        data.nft_approval_id,
        data.accepted_offer_id,
        &rt.pg_connection,
    )
    .await
    {
        rt.minterop_rpc
            .sale(
                data.nft_contract_id.to_string(),
                data.nft_token_id,
                offerer,
                tx.id,
            )
            .await;
    }
}
