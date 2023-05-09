use mb_sdk::events::market_v1::NftSaleData;

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

    diesel::update(
        dsl::nft_listings
            .filter(dsl::token_id.eq(token_id.to_string()))
            .filter(dsl::nft_contract_id.eq(nft_contract.to_string()))
            .filter(dsl::approval_id.eq(pg_numeric(approval_id))),
    )
    .set((
        dsl::accepted_at.eq(tx.timestamp),
        dsl::accepted_offer_id.eq(data.offer_num as i64),
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

    diesel::update(
        dsl::nft_offers
            .filter(dsl::token_id.eq(token_id.to_string()))
            .filter(dsl::nft_contract_id.eq(nft_contract.to_string()))
            .filter(dsl::approval_id.eq(pg_numeric(approval_id)))
            .filter(dsl::offer_id.eq(data.offer_num as i64)),
    )
    .set((dsl::accepted_at.eq(tx.timestamp),))
    .execute_db(&rt.pg_connection, &tx, "update offer on sale")
    .await
}

async fn insert_nft_earnings(
    rt: TxProcessingRuntime,
    tx: ReceiptData,
    mut data: NftSaleData,
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

    let mut values = data
        .payout
        .drain()
        .map(|(receiver_id, amount)| NftEarning {
            token_id: token_id.to_string(),
            nft_contract_id: nft_contract.to_string(),
            market_id: tx.receiver.to_string(),
            approval_id: pg_numeric(approval_id),
            offer_id: data.offer_num as i64,
            receipt_id: tx.id.clone(),
            timestamp: tx.timestamp,
            currency: "near".to_string(),
            receiver_id: receiver_id.to_string(),
            amount: pg_numeric(amount.0),
            is_referral: false,
            is_affiliate: false,
            is_mintbase_cut: false,
        })
        .collect::<Vec<_>>();

    if let Some(mb_amount) = data.mintbase_amount {
        values.push(NftEarning {
            token_id: token_id.to_string(),
            nft_contract_id: nft_contract.to_string(),
            market_id: tx.receiver.to_string(),
            approval_id: pg_numeric(approval_id),
            offer_id: data.offer_num as i64,
            receipt_id: tx.id.clone(),
            timestamp: tx.timestamp,
            currency: "near".to_string(),
            receiver_id: tx.receiver.to_string(),
            amount: pg_numeric(mb_amount.0),
            is_referral: false,
            is_affiliate: false,
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

    if let (lister_currency, Some(offerer)) =
        crate::database::query_lister_currency_offerer(
            nft_contract.to_string(),
            token_id.to_string(),
            tx.receiver.to_string(),
            approval_id,
            data.offer_num,
            &rt.pg_connection,
        )
        .await
    {
        let lister = lister_currency.map(|lc| lc.0);
        // FIXME: implicit assumption: cut is 2.5%
        let price =
            data.payout.values().fold(0, |acc, el| acc + el.0) / 975 * 1000;

        let activity = NftActivity {
            receipt_id: tx.id.clone(),
            tx_sender: tx.sender.to_string(),
            sender_pk: tx.sender_pk.clone(),
            timestamp: tx.timestamp,
            nft_contract_id: nft_contract.to_string(),
            token_id: token_id.to_string(),
            kind: NFT_ACTIVITY_KIND_SOLD.to_string(),
            action_sender: offerer,
            action_receiver: lister,
            memo: None,
            price: Some(pg_numeric(price)),
            currency: Some(CURRENCY_NEAR.to_string()),
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

    diesel::update(
        nft_listings::table
            .filter(dsl::nft_contract_id.eq(nft_contract.to_string()))
            .filter(dsl::token_id.eq(token_id.to_string()))
            .filter(dsl::approval_id.eq(pg_numeric(approval_id)))
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

    diesel::update(
        nft_offers::table
            .filter(dsl::nft_contract_id.eq(nft_contract.to_string()))
            .filter(dsl::token_id.eq(token_id.to_string()))
            .filter(dsl::approval_id.eq(pg_numeric(approval_id)))
            .filter(dsl::market_id.eq(tx.receiver.to_string()))
            .filter(dsl::offer_id.eq(data.offer_num as i64)),
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
    let (nft_contract_id, token_id, approval_id) =
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

    if let Some(offerer) = crate::database::query_offerer(
        nft_contract_id.to_string(),
        token_id.to_string(),
        tx.receiver.to_string(),
        approval_id,
        data.offer_num,
        &rt.pg_connection,
    )
    .await
    {
        rt.minterop_rpc
            .sale(
                nft_contract_id.to_string(),
                token_id.to_string(),
                offerer,
                tx.id,
            )
            .await;
    }
}
