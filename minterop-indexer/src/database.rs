// ------------------------------ actix_diesel ------------------------------ //
pub(crate) type DbConnPool = actix_diesel::Database<diesel::PgConnection>;

// timeouts and lifetimes?
// https://docs.rs/actix-diesel/0.3.0/actix_diesel/struct.Builder.html
pub(crate) fn init_db_connection(pg_string: &str) -> DbConnPool {
    actix_diesel::Database::builder()
        .pool_max_size(20)
        .open(pg_string)
}

#[async_trait::async_trait]
pub(crate) trait ExecuteDb {
    async fn execute_db(self, db: &DbConnPool, tx: &crate::runtime::ReceiptData, msg: &str);
}

#[async_trait::async_trait]
impl<Q> ExecuteDb for Q
where
    Q: actix_diesel::dsl::AsyncRunQueryDsl<diesel::PgConnection>
        + diesel::query_dsl::load_dsl::ExecuteDsl<diesel::PgConnection>
        + Send,
{
    async fn execute_db(self, db: &DbConnPool, tx: &crate::runtime::ReceiptData, msg: &str) {
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
    use diesel::{ExpressionMethods, QueryDsl};
    use minterop_data::schema::nft_tokens::dsl;

    match dsl::nft_tokens
        .filter(dsl::nft_contract_id.eq(nft_contract_id))
        .filter(dsl::token_id.eq(token_id))
        .select(dsl::metadata_id)
        .limit(1)
        .get_results_async::<Option<String>>(db)
        .await
    {
        Err(_) => None,
        Ok(values) if values.len() > 0 => match values.get(0) {
            Some(Some(s)) => Some(s.to_string()),
            _ => None,
        },
        Ok(_) => None,
    }
}
