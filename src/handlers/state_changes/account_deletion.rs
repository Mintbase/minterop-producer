use actix_diesel::dsl::AsyncRunQueryDsl;
use chrono::NaiveDateTime;
use near_lake_framework::near_indexer_primitives::views::StateChangeValueView;

use crate::{
    handlers::prelude::*,
    logging::HandleErr,
};

pub(crate) async fn handle_account_deletion(
    rt: &TxProcessingRuntime,
    timestamp: NaiveDateTime,
    state_change_value: StateChangeValueView,
) {
    use minterop_data::schema::accounts::dsl;

    match state_change_value {
        StateChangeValueView::AccountDeletion { account_id } => 
            diesel::update(accounts::table)
                .filter(dsl::account_id.eq(account_id.to_string()))
                .filter(dsl::removed_at.is_null())
                .set(dsl::removed_at.eq(timestamp))
                .execute_async(&rt.pg_connection)
                .await
                .map(|updated| {
                    if updated != 1 {
                        crate::error!(
                            "Expected to update 1 row, updated {} instead.(account_id: {}, timestamp: {})",
                            updated, account_id.as_str(), timestamp
                        );
                    }
                }).handle_err(|err|{crate::error!("Failed to delete account: {:?}", err)}),
        _ => {
            crate::warn!("Could not handle account deletion.");
            return;
        }
    };
}
