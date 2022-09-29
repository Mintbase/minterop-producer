use mb_sdk::events::nft_payouts::NftSetSplitOwnerData;

use crate::handlers::prelude::*;

pub(crate) async fn handle_nft_set_split_owners(
    rt: &TxProcessingRuntime,
    tx: &ReceiptData,
    data: serde_json::Value,
) {
    use minterop_data::schema::nft_tokens::dsl;

    let token_ids = match serde_json::from_value::<NftSetSplitOwnerData>(data.clone()) {
        Err(_) => {
            error!(r#"Invalid log for "nft_transfer": {} ({:?})"#, data, tx);
            return;
        }
        Ok(data) => data.token_ids,
    };

    // unwrap ok, because schema has been validated
    let splits_json = data.get("split_owners").unwrap();

    // TODO: can this be accomplished in a single query?
    future::join_all(token_ids.into_iter().map(|token_id| {
        diesel::update(
            nft_tokens::table
                .filter(dsl::nft_contract_id.eq(tx.receiver.to_string()))
                .filter(dsl::token_id.eq(token_id)),
        )
        .set(dsl::splits.eq(splits_json.clone()))
        .execute_db(&rt.pg_connection, tx, "set splits")
    }))
    .await;
}
