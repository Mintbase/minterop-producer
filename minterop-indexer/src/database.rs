// ------------------------------ tokio_diesel ------------------------------ //
// pub(crate) type DbConnPool =
//     diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>;
// // timeouts and lifetimes?
// // https://docs.diesel.rs/diesel/r2d2/struct.Builder.html
// pub(crate) fn init_db_connection(pg_string: &str) -> DbConnPool {
//     let pg_mgr = diesel::r2d2::ConnectionManager::new(&self.postgres_string);
//     let pg_connection = diesel::r2d2::Pool::builder()
//         .max_size(20)
//         .build(pg_mgr)
//         .unwrap();
// }

// ------------------------------ actix_diesel ------------------------------ //
pub(crate) type DbConnPool = actix_diesel::Database<diesel::PgConnection>;

// timeouts and lifetimes?
// https://docs.rs/actix-diesel/0.3.0/actix_diesel/struct.Builder.html
pub(crate) fn init_db_connection(pg_string: &str) -> DbConnPool {
    actix_diesel::Database::builder()
        .pool_max_size(20)
        .open(pg_string)
}

// pub(crate) type AsyncQueryResult = std::pin::Pin<
//     Box<
//         dyn futures::Future<
//             Output = Result<
//                 (),
//                 actix_diesel::AsyncError<diesel::result::Error>,
//             >,
//         >,
//     >,
// >;

// pub(crate) type AsyncQuery =
//     std::pin::Pin<Box<dyn futures::Future<Output = ()>>>;

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
