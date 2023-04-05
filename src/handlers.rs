pub(crate) mod prelude {
    pub use diesel::{
        ExpressionMethods,
        QueryDsl,
    };
    pub use futures::future;
    pub use minterop_data::{
        db_rows::*,
        pg_numeric,
        schema::*,
    };

    pub(crate) use crate::{
        database::ExecuteDb,
        error,
        runtime::TxProcessingRuntime,
        ReceiptData,
    };
}

crate::forward_mod!(nft_core);
crate::forward_mod!(nft_approvals);
crate::forward_mod!(nft_payouts);
crate::forward_mod!(mb_store_settings);
crate::forward_mod!(contract_metadata_update);

pub mod market_v01;
pub mod market_v02;

async fn invalidate_nft_listings(
    rt: crate::runtime::TxProcessingRuntime,
    tx: crate::ReceiptData,
    nft_contract_id: String,
    token_ids: Vec<String>,
    market_id: Option<String>,
) {
    use diesel::dsl::any;
    use minterop_data::schema::nft_listings::dsl;

    use crate::handlers::prelude::*;

    let source = nft_listings::table
        .filter(dsl::nft_contract_id.eq(nft_contract_id))
        .filter(dsl::token_id.eq(any(token_ids)))
        .filter(dsl::accepted_at.is_null())
        .filter(dsl::unlisted_at.is_null())
        .filter(dsl::invalidated_at.is_null());

    if let Some(market_id) = market_id {
        diesel::update(source.filter(dsl::market_id.eq(market_id)))
            .set(dsl::invalidated_at.eq(tx.timestamp))
            .execute_db(&rt.pg_connection, &tx, "invalidate listing")
            .await
    } else {
        diesel::update(source)
            .set(dsl::invalidated_at.eq(tx.timestamp))
            .execute_db(&rt.pg_connection, &tx, "invalidate listing")
            .await
    }
}

async fn invalidate_nft_offers(
    rt: crate::runtime::TxProcessingRuntime,
    tx: crate::ReceiptData,
    nft_contract_id: String,
    token_ids: Vec<String>,
    market_id: Option<String>,
) {
    use diesel::dsl::any;
    use minterop_data::schema::nft_offers::dsl;

    use crate::handlers::prelude::*;

    let source = nft_offers::table
        .filter(dsl::nft_contract_id.eq(nft_contract_id))
        .filter(dsl::token_id.eq(any(token_ids)))
        .filter(dsl::accepted_at.is_null())
        .filter(dsl::withdrawn_at.is_null())
        .filter(dsl::outbid_at.is_null())
        .filter(dsl::invalidated_at.is_null());

    if let Some(market_id) = market_id {
        diesel::update(source.filter(dsl::market_id.eq(market_id)))
            .set(dsl::invalidated_at.eq(tx.timestamp))
            .execute_db(&rt.pg_connection, &tx, "invalidate offer")
            .await
    } else {
        diesel::update(source)
            .set(dsl::invalidated_at.eq(tx.timestamp))
            .execute_db(&rt.pg_connection, &tx, "invalidate offer")
            .await
    }
}
