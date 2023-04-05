use serde::Deserialize;

use crate::runtime::{
    ReceiptData,
    TxProcessingRuntime,
};

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
            handle_resolve_purchase_fail(rt, tx, log).await
        }
        "resolve_purchase" => handle_resolve_purchase(rt, tx, log).await,
        "add_offer" => handle_add_offer(rt, tx, log).await,
        "delete_offer" => handle_delete_offer(rt, tx, log).await,
        "add_trade" => { /* not following */ }
        "delete_trade" => { /* not following */ }
        "accept_trade" => { /* not following */ }
        "extend_auction" => { /* not following */ }
        "add_bid" => handle_add_bid(rt, tx, log).await,
        "cancel_bid" => handle_cancel_bid(rt, tx, log).await,
        "add_market_data" => handle_add_market_data(rt, tx, log).await,
        "delete_market_data" => handle_delete_market_data(rt, tx, log).await,
        _ => crate::warn!("Paras implemented a new marketplace event: {}", log),
    }
}

/// nft_transfer_payout failed
async fn handle_resolve_purchase_fail(
    rt: &TxProcessingRuntime,
    tx: &ReceiptData,
    log: &str,
) {
    // TODO: actually handling this
}

/// nft_transfer_payout succeeded
async fn handle_resolve_purchase(
    rt: &TxProcessingRuntime,
    tx: &ReceiptData,
    log: &str,
) {
    // TODO: actually handling this
}

async fn handle_add_offer(
    rt: &TxProcessingRuntime,
    tx: &ReceiptData,
    log: &str,
) {
    // TODO: actually handling this
}

async fn handle_delete_offer(
    rt: &TxProcessingRuntime,
    tx: &ReceiptData,
    log: &str,
) {
    // TODO: actually handling this
}

/// Create auction offer
async fn handle_add_bid(rt: &TxProcessingRuntime, tx: &ReceiptData, log: &str) {
    // TODO: actually handling this
}

/// Remove auction offer
async fn handle_cancel_bid(
    rt: &TxProcessingRuntime,
    tx: &ReceiptData,
    log: &str,
) {
    // TODO: actually handling this
}

/// Create a listing on paras
async fn handle_add_market_data(
    rt: &TxProcessingRuntime,
    tx: &ReceiptData,
    log: &str,
) {
    // TODO: actually handling this
}

/// Remove a listing on paras
async fn handle_delete_market_data(
    rt: &TxProcessingRuntime,
    tx: &ReceiptData,
    log: &str,
) {
    // TODO: actually handling this
}
