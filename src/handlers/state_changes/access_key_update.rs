use actix_diesel::dsl::AsyncRunQueryDsl;
use chrono::NaiveDateTime;
use near_lake_framework::near_indexer_primitives::views::StateChangeValueView;

use crate::{
    handlers::prelude::*,
    logging::HandleErr,
};

pub(crate) async fn handle_access_key_update(
    rt: &TxProcessingRuntime,
    timestamp: NaiveDateTime,
    state_change_value: StateChangeValueView,
) {
    let key_update: AccessKey = match state_change_value {
        // FIXME: skip if this is not a FullAccessKey
        StateChangeValueView::AccessKeyUpdate {
            account_id,
            public_key,
            access_key: _,
        } => AccessKey {
            account_id: account_id.to_string(),
            public_key: public_key.to_string(),
            created_at: timestamp,
            removed_at: None,
        },
        other => {
            crate::warn!("Could not handle access key update. Expected `StateChangeValueView::AccessKeyUpdate but got `{:?}` instead.", other);
            return;
        }
    };

    diesel::insert_into(access_keys::table)
        .values(key_update)
        .execute_async(&rt.pg_connection)
        .await
        .handle_err(|err| {
            crate::error!("Failed to update access_key: {:?}", err)
        });
}
