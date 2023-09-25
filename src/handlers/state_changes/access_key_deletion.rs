use actix_diesel::dsl::AsyncRunQueryDsl;
use chrono::NaiveDateTime;
use near_lake_framework::near_indexer_primitives::views::StateChangeValueView;

use crate::{
    handlers::prelude::*,
    logging::HandleErr,
};

pub(crate) async fn handle_access_key_deletion(
    rt: &TxProcessingRuntime,
    timestamp: NaiveDateTime,
    state_change_value: StateChangeValueView,
) {
    use minterop_data::schema::access_keys::dsl;

    let (account_id, public_key) = match state_change_value {
        StateChangeValueView::AccessKeyDeletion {
            account_id,
            public_key,
        } => (account_id, public_key),
        other => {
            crate::warn!("Could not handle access key deletion. Expected `StateChangeValueView::AccessKeyDeletion but got `{:?}` instead.", other);
            return;
        }
    };

    diesel::update(access_keys::table)
    .filter(dsl::account_id.eq(account_id.to_string()))
    .filter(dsl::public_key.eq(public_key.to_string()))
    .filter(dsl::removed_at.is_null())
    .set(dsl::removed_at.eq(timestamp))
    .execute_async(&rt.pg_connection)
    .await
    .map(|updated| {
        if updated != 1 {
            crate::error!(
                "Expected to update 1 row, updated {} instead.(account_id: {}, public_key: {} timestamp: {})",
                updated, account_id.as_str(), public_key.to_string(), timestamp
            );
        }
    }).handle_err(|err| crate::error!("Failed to delete access_key: {:?}", err));
}
