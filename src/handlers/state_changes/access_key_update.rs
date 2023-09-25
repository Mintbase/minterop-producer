use actix_diesel::dsl::AsyncRunQueryDsl;
use chrono::NaiveDateTime;
use near_lake_framework::near_indexer_primitives::views::{
    AccessKeyPermissionView,
    StateChangeValueView,
};

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
        StateChangeValueView::AccessKeyUpdate {
            account_id,
            public_key,
            access_key,
        } => AccessKey {
            account_id: account_id.to_string(),
            public_key: public_key.to_string(),
            permissions: match access_key.permission {
                p @ AccessKeyPermissionView::FunctionCall { .. } => {
                    Some(match serde_json::to_value(p) {
                        Ok(mut val) => val
                            .as_object_mut()
                            .unwrap()
                            .remove("FunctionCall")
                            .unwrap(),
                        Err(e) => {
                            crate::error!("Failed to partially serialize the AccessKeyPermissionView: {}", e);
                            return;
                        }
                    })
                }
                AccessKeyPermissionView::FullAccess => None,
            },
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
