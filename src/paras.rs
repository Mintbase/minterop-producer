use serde::Deserialize;

use crate::handlers::prelude::*;

#[derive(Deserialize)]
struct ParasMarketEvent {
    #[serde(rename = "type")]
    _type: String,
    params: serde_json::Value,
}

const UNSTRUCTURED_LOG_PREFIXES: [&str; 3] = [
    "Paras: Offer does not exist",
    "Paras: seller's nft failed to trade, rollback buyer's nft",
    "Insufficient storage paid: ",
];

fn is_unstructured_log_prefix(log: &str) -> bool {
    for prefix in UNSTRUCTURED_LOG_PREFIXES {
        if log.starts_with(prefix) {
            return true;
        }
    }
    false
}

#[derive(Deserialize, Debug)]
struct AddMarketDataParams {
    owner_id: String,
    approval_id: u64,
    nft_contract_id: String,
    token_id: String,
    ft_token_id: String,
    price: near_sdk::json_types::U128,
    #[allow(unused)]
    started_at: Option<u64>,
    #[allow(unused)]
    ended_at: Option<near_sdk::json_types::U64>,
    #[allow(unused)]
    end_price: Option<near_sdk::json_types::U128>,
    #[allow(unused)]
    is_auction: Option<bool>,
    #[allow(unused)]
    transaction_fee: near_sdk::json_types::U128,
}

#[derive(Deserialize, Debug)]
struct DeleteMarketDataParams {
    owner_id: String,
    nft_contract_id: String,
    token_id: String,
}

#[derive(Deserialize, Debug)]
struct ResolvePurchaseParams {
    owner_id: String,
    nft_contract_id: String,
    token_id: String,
    #[allow(unused)]
    token_series_id: Option<String>,
    #[allow(unused)]
    ft_token_id: String,
    price: near_sdk::json_types::U128,
    buyer_id: String,
    #[allow(unused)]
    is_offer: Option<bool>,
}

pub(crate) async fn handle_paras_market_log(
    rt: &TxProcessingRuntime,
    tx: &ReceiptData,
    log: &str,
) {
    let event = match serde_json::from_str::<ParasMarketEvent>(log) {
        Ok(event) => event,
        Err(e) => {
            if !is_unstructured_log_prefix(log) {
                crate::error!(
                    "Error deserializing paras event {}: {} ({:?})",
                    log,
                    e,
                    tx
                );
            }
            return;
        }
    };

    match event._type.as_str() {
        "resolve_purchase_fail" => {
            handle_resolve_purchase_fail(rt, tx, event.params).await
        }
        "resolve_purchase" => {
            handle_resolve_purchase(rt, tx, event.params).await
        }
        "add_offer" => { /* not following */ }
        "delete_offer" => { /* not following */ }
        "add_trade" => { /* not following */ }
        "delete_trade" => { /* not following */ }
        "accept_trade" => { /* not following */ }
        "extend_auction" => { /* not following */ }
        "add_bid" => { /* not following */ }
        "cancel_bid" => { /* not following */ }
        "add_market_data" => handle_add_market_data(rt, tx, event.params).await,
        "delete_market_data" => {
            handle_delete_market_data(rt, tx, event.params).await
        }
        _ => crate::warn!("Paras implemented a new marketplace event: {}", log),
    }
}

/// nft_transfer_payout failed
async fn handle_resolve_purchase_fail(
    rt: &TxProcessingRuntime,
    tx: &ReceiptData,
    params: serde_json::Value,
) {
    use nft_external_listings::dsl;

    let params =
        match serde_json::from_value::<ResolvePurchaseParams>(params.clone()) {
            Ok(params) => params,
            Err(e) => {
                crate::error!(
                    "Paras market params structure changed: {} ({:?}, {:?})",
                    e,
                    params,
                    tx
                );
                return;
            }
        };

    diesel::update(
        dsl::nft_external_listings
            .filter(dsl::nft_contract_id.eq(params.nft_contract_id))
            .filter(dsl::token_id.eq(params.token_id))
            .filter(dsl::market_id.eq(tx.receiver.to_string()))
            // weird to use lister instead of approval_id, but that's what we get
            .filter(dsl::lister_id.eq(params.owner_id)),
    )
    .set((
        dsl::failed_at.eq(tx.timestamp),
        dsl::failure_receipt_id.eq(tx.id.clone()),
    ))
    .execute_db(&rt.pg_connection, tx, "mark external listing as failed")
    .await;
}

/// nft_transfer_payout succeeded
async fn handle_resolve_purchase(
    rt: &TxProcessingRuntime,
    tx: &ReceiptData,
    params: serde_json::Value,
) {
    use nft_external_listings::dsl;

    let params =
        match serde_json::from_value::<ResolvePurchaseParams>(params.clone()) {
            Ok(params) => params,
            Err(e) => {
                crate::error!(
                    "Paras market params structure changed: {} ({:?}, {:?})",
                    e,
                    params,
                    tx
                );
                return;
            }
        };

    diesel::update(
        dsl::nft_external_listings
            .filter(dsl::nft_contract_id.eq(params.nft_contract_id))
            .filter(dsl::token_id.eq(params.token_id))
            .filter(dsl::market_id.eq(tx.receiver.to_string()))
            // weird to use lister instead of approval_id, but that's what we get
            .filter(dsl::lister_id.eq(params.owner_id)),
    )
    .set((
        dsl::buyer_id.eq(params.buyer_id),
        dsl::sale_price.eq(pg_numeric(params.price.0)),
        dsl::sold_at.eq(tx.timestamp),
        dsl::sale_receipt_id.eq(tx.id.clone()),
    ))
    .execute_db(&rt.pg_connection, tx, "mark external listing as sold")
    .await;
}

/// Create a listing on paras
async fn handle_add_market_data(
    rt: &TxProcessingRuntime,
    tx: &ReceiptData,
    params: serde_json::Value,
) {
    let params =
        match serde_json::from_value::<AddMarketDataParams>(params.clone()) {
            Ok(params) => params,
            Err(e) => {
                crate::error!(
                    "Paras market params structure changed: {} ({:?}, {:?})",
                    e,
                    params,
                    tx
                );
                return;
            }
        };

    let currency = if params.ft_token_id.as_str() == "near" {
        params.ft_token_id
    } else {
        format!("ft::{}", params.ft_token_id)
    };

    diesel::insert_into(nft_external_listings::table)
        .values(NftExternalListing {
            nft_contract_id: params.nft_contract_id,
            token_id: params.token_id,
            market_id: tx.receiver.to_string(),
            approval_id: pg_numeric(params.approval_id),
            lister_id: params.owner_id.to_string(),
            listing_price: pg_numeric(params.price.0),
            listed_at: tx.timestamp,
            listing_receipt_id: tx.id.clone(),
            currency,
            buyer_id: None,
            sale_price: None,
            sold_at: None,
            sale_receipt_id: None,
            deleted_at: None,
            deletion_receipt_id: None,
            failed_at: None,
            failure_receipt_id: None,
        })
        .execute_db(&rt.pg_connection, tx, "insert external listing")
        .await;
}

/// Remove a listing on paras
async fn handle_delete_market_data(
    rt: &TxProcessingRuntime,
    tx: &ReceiptData,
    params: serde_json::Value,
) {
    use nft_external_listings::dsl;

    let params = match serde_json::from_value::<DeleteMarketDataParams>(
        params.clone(),
    ) {
        Ok(params) => params,
        Err(e) => {
            crate::error!(
                "Paras market params structure changed: {} ({:?}, {:?})",
                e,
                params,
                tx
            );
            return;
        }
    };
    diesel::update(
        dsl::nft_external_listings
            .filter(dsl::nft_contract_id.eq(params.nft_contract_id))
            .filter(dsl::token_id.eq(params.token_id))
            .filter(dsl::market_id.eq(tx.receiver.to_string()))
            // weird to use lister instead of approval_id, but that's what we get
            .filter(dsl::lister_id.eq(params.owner_id)),
    )
    .set((
        dsl::deleted_at.eq(tx.timestamp),
        dsl::deletion_receipt_id.eq(tx.id.clone()),
    ))
    .execute_db(&rt.pg_connection, tx, "mark external listing as deleted")
    .await;
}
