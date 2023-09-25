use actix_diesel::dsl::AsyncRunQueryDsl;
use chrono::NaiveDateTime;
use near_lake_framework::near_indexer_primitives::views::StateChangeValueView;

use crate::{
    handlers::prelude::*,
    logging::HandleErr,
};

pub(crate) async fn handle_account_update(
    rt: &TxProcessingRuntime,
    timestamp: NaiveDateTime,
    state_change_value: StateChangeValueView,
) {
    use minterop_data::schema::accounts::dsl;

    let account: Account = match state_change_value {
        StateChangeValueView::AccountUpdate {
            account_id,
            account,
        } => Account {
            account_id: account_id.to_string(),
            amount: account.amount.to_string(),
            locked: account.locked.to_string(),
            code_hash: account.code_hash.to_string(),
            storage_usage: pg_numeric(account.storage_usage),
            storage_paid_at: pg_numeric(account.storage_paid_at),
            created_at: timestamp,
            removed_at: None,
        },
        _ => {
            crate::warn!("Could not handle account update.");
            return;
        }
    };

    diesel::insert_into(accounts::table)
        .values(account.clone())
        .on_conflict(diesel::pg::upsert::on_constraint("accounts_pkey"))
        .do_update()
        .set((
            dsl::amount.eq(account.amount),
            dsl::locked.eq(account.locked),
            dsl::code_hash.eq(account.code_hash),
            dsl::storage_usage.eq(account.storage_usage),
            dsl::storage_paid_at.eq(account.storage_paid_at),
        ))
        .execute_async(&rt.pg_connection)
        .await
        .handle_err(|err| crate::error!("Failed to update account: {:?}", err));
}
