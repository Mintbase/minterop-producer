use crate::handlers::prelude::*;

// FIXME: copied from mb-contracts, this repo should eventually point there
#[derive(serde::Deserialize, Clone, Debug)]
pub struct NftFailedListingData {
    pub nft_contract_id: near_sdk::AccountId,
    pub nft_token_id: String,
    pub nft_approval_id: u64,
    pub offer_id: u64,
}

pub(crate) async fn handle_nft_failed_listing(
    rt: &TxProcessingRuntime,
    tx: &ReceiptData,
    data: serde_json::Value,
) {
    let data =
        match serde_json::from_value::<NftFailedListingData>(data.clone()) {
            Err(_) => {
                error!(r#"Invalid log for "nft_list": {} ({:?})"#, data, tx);
                return;
            }
            Ok(data) => data,
        };

    update_nft_listings(rt.clone(), tx.clone(), data.clone()).await;
}

async fn update_nft_listings(
    rt: TxProcessingRuntime,
    tx: ReceiptData,
    data: NftFailedListingData,
) {
    use nft_listings::dsl;

    diesel::update(
        dsl::nft_listings
            .filter(dsl::token_id.eq(data.nft_token_id.to_string()))
            .filter(dsl::nft_contract_id.eq(data.nft_contract_id.to_string()))
            .filter(dsl::market_id.eq(tx.receiver.to_string()))
            .filter(dsl::approval_id.eq(pg_numeric(data.nft_approval_id))),
    )
    .set(dsl::invalidated_at.eq(tx.timestamp))
    .execute_db(&rt.pg_connection, &tx, "update listing on unlist")
    .await
}
