use actix_diesel::dsl::AsyncRunQueryDsl;
use chrono::NaiveDateTime;
use diesel::sql_types::Jsonb;
use near_lake_framework::near_indexer_primitives::{
    types::AccountId,
    views::{
        AccessKeyPermissionView,
        AccessKeyView,
        StateChangeValueView,
    },
};

use crate::handlers::prelude::*;

pub(crate) async fn handle_access_key_deletion(
    state_change_value: StateChangeValueView,
    timestamp: NaiveDateTime,
    rt: TxProcessingRuntime,
) {
    use minterop_data::schema::access_keys::dsl;
    match state_change_value {
        StateChangeValueView::AccessKeyDeletion {
            account_id,
            public_key
        } => diesel::update(access_keys::table)
            .filter(dsl::account_id.eq(account_id.to_string()))
            .set(dsl::removed_at.eq(timestamp))
            .execute_async(&rt.pg_connection),
        _ => {
            crate::warn!("Could not handle access key deletion.");
            return;
        }
    };
}
