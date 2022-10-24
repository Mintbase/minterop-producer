crate::forward_mod!(nft_mint);
crate::forward_mod!(nft_transfer);
crate::forward_mod!(nft_burn);

async fn invalidate_nft_listings(
    rt: crate::runtime::TxProcessingRuntime,
    tx: crate::ReceiptData,
    token_ids: Vec<String>,
) {
    use diesel::dsl::any;
    use minterop_data::schema::nft_listings::dsl;

    use crate::handlers::prelude::*;

    diesel::update(
        nft_listings::table
            .filter(dsl::nft_contract_id.eq(tx.receiver.to_string()))
            .filter(dsl::token_id.eq(any(token_ids)))
            .filter(dsl::accepted_at.is_null())
            .filter(dsl::unlisted_at.is_null())
            .filter(dsl::invalidated_at.is_null()),
    )
    .set(dsl::invalidated_at.eq(tx.timestamp))
    .execute_db(&rt.pg_connection, &tx, "invalidate listing")
    .await
}

async fn invalidate_nft_offers(
    rt: crate::runtime::TxProcessingRuntime,
    tx: crate::ReceiptData,
    token_ids: Vec<String>,
) {
    use diesel::dsl::any;
    use minterop_data::schema::nft_offers::dsl;

    use crate::handlers::prelude::*;

    diesel::update(
        nft_offers::table
            .filter(dsl::nft_contract_id.eq(tx.receiver.to_string()))
            .filter(dsl::token_id.eq(any(token_ids)))
            .filter(dsl::accepted_at.is_null())
            .filter(dsl::withdrawn_at.is_null())
            .filter(dsl::outbid_at.is_null())
            .filter(dsl::invalidated_at.is_null()),
    )
    .set(dsl::invalidated_at.eq(tx.timestamp))
    .execute_db(&rt.pg_connection, &tx, "invalidate offer")
    .await
}
