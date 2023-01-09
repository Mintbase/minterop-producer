const DEFAULT_DB_POOL_SIZE: u32 = 50;

// ------------------------------ actix_diesel ------------------------------ //
pub(crate) type DbConnPool = actix_diesel::Database<diesel::PgConnection>;

// timeouts and lifetimes?
// https://docs.rs/actix-diesel/0.3.0/actix_diesel/struct.Builder.html
pub(crate) fn init_db_connection(
    pg_string: &str,
    db_pool_size: Option<u32>,
) -> DbConnPool {
    actix_diesel::Database::builder()
        .pool_max_size(db_pool_size.unwrap_or(DEFAULT_DB_POOL_SIZE))
        .open(pg_string)
}

#[async_trait::async_trait]
pub(crate) trait ExecuteDb {
    async fn execute_db(
        self,
        db: &DbConnPool,
        tx: &crate::runtime::ReceiptData,
        msg: &str,
    );
}

#[async_trait::async_trait]
impl<Q> ExecuteDb for Q
where
    Q: actix_diesel::dsl::AsyncRunQueryDsl<diesel::PgConnection>
        + diesel::query_dsl::load_dsl::ExecuteDsl<diesel::PgConnection>
        + Send,
{
    async fn execute_db(
        self,
        db: &DbConnPool,
        tx: &crate::runtime::ReceiptData,
        msg: &str,
    ) {
        if let Err(e) = self.execute_async(db).await {
            crate::error!("Failed to {}: {} ({:?})", msg, e, tx);
        }
    }
}

pub(crate) async fn query_metadata_id(
    nft_contract_id: String,
    token_id: String,
    db: &DbConnPool,
) -> Option<String> {
    use actix_diesel::dsl::AsyncRunQueryDsl;
    use diesel::{
        ExpressionMethods,
        QueryDsl,
    };
    use minterop_data::schema::nft_tokens::dsl;

    match dsl::nft_tokens
        .filter(dsl::nft_contract_id.eq(nft_contract_id))
        .filter(dsl::token_id.eq(token_id))
        .select(dsl::metadata_id)
        .limit(1)
        .get_results_async::<Option<String>>(db)
        .await
    {
        Err(e) => {
            crate::error!("Failed to query metadata ID: {}", e);
            None
        }
        Ok(values) if !values.is_empty() => match values.get(0) {
            Some(Some(s)) => Some(s.to_string()),
            _ => None,
        },
        Ok(_) => None,
    }
}

pub(crate) async fn query_lister(
    nft_contract_id: String,
    token_id: String,
    market_id: String,
    approval_id: u64,
    db: &DbConnPool,
) -> Option<String> {
    use actix_diesel::dsl::AsyncRunQueryDsl;
    use diesel::{
        ExpressionMethods,
        QueryDsl,
    };
    use minterop_data::{
        pg_numeric,
        schema::nft_listings::{
            self,
            dsl as listings_dsl,
        },
    };

    match nft_listings::table
        .filter(listings_dsl::nft_contract_id.eq(nft_contract_id))
        .filter(listings_dsl::token_id.eq(token_id.clone()))
        .filter(listings_dsl::market_id.eq(market_id))
        .filter(listings_dsl::approval_id.eq(pg_numeric(approval_id)))
        .select(listings_dsl::listed_by)
        .limit(1)
        .get_result_async::<String>(&db)
        .await
    {
        Err(e) => {
            crate::error!("Failed to get token lister: {}", e);
            None
        }
        Ok(lister) => Some(lister),
    }
}

pub(crate) async fn query_offerer(
    nft_contract_id: String,
    token_id: String,
    market_id: String,
    approval_id: u64,
    offer_id: u64,
    db: &DbConnPool,
) -> Option<String> {
    use actix_diesel::dsl::AsyncRunQueryDsl;
    use diesel::{
        ExpressionMethods,
        QueryDsl,
    };
    use minterop_data::{
        pg_numeric,
        schema::nft_offers::{
            self,
            dsl as offers_dsl,
        },
    };

    match nft_offers::table
        .filter(offers_dsl::nft_contract_id.eq(nft_contract_id))
        .filter(offers_dsl::token_id.eq(token_id))
        .filter(offers_dsl::market_id.eq(market_id))
        .filter(offers_dsl::approval_id.eq(pg_numeric(approval_id)))
        .filter(offers_dsl::offer_id.eq(offer_id as i64))
        .select(offers_dsl::offered_by)
        .limit(1)
        .get_result_async::<String>(&db)
        .await
    {
        Err(e) => {
            crate::error!("Failed to get offerer {}", e);
            None
        }
        Ok(offerer) => Some(offerer),
    }
}

pub(crate) async fn query_lister_and_offerer(
    nft_contract_id: String,
    token_id: String,
    market_id: String,
    approval_id: u64,
    offer_id: u64,
    db: &DbConnPool,
) -> (Option<String>, Option<String>) {
    let lister = query_lister(
        nft_contract_id.clone(),
        token_id.clone(),
        market_id.clone(),
        approval_id,
        db,
    )
    .await;

    let offerer = query_offerer(
        nft_contract_id,
        token_id,
        market_id,
        approval_id,
        offer_id,
        db,
    )
    .await;

    (lister, offerer)
}
