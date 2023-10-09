use chrono::NaiveDateTime;
use itertools::Itertools;
use near_crypto::PublicKey;
use near_lake_framework::near_indexer_primitives::{
    types::AccountId,
    views::StateChangeValueView,
};

use crate::runtime::TxProcessingRuntime;

pub struct StateChangeAggregator {
    // (account_id, public_key, timestamp)
    key_additions: Vec<(AccountId, PublicKey)>,
    key_deletions: Vec<(AccountId, PublicKey)>,
    account_creations: Vec<AccountId>,
    account_deletions: Vec<AccountId>,
}

impl From<Vec<StateChangeValueView>> for StateChangeAggregator {
    fn from(state_changes: Vec<StateChangeValueView>) -> StateChangeAggregator {
        use near_lake_framework::near_indexer_primitives::views::AccessKeyPermissionView;

        let mut key_additions = Vec::with_capacity(state_changes.len() / 2);
        let mut key_deletions = Vec::with_capacity(state_changes.len() / 2);
        // let mut account_creations = Vec::with_capacity(state_changes.len() / 2);
        // let mut account_deletions = Vec::with_capacity(state_changes.len() / 2);
        let account_creations = Vec::new();
        let account_deletions = Vec::new();

        for sc in state_changes {
            match sc {
                StateChangeValueView::AccessKeyUpdate {
                    account_id,
                    public_key,
                    access_key,
                } => {
                    if let AccessKeyPermissionView::FullAccess =
                        access_key.permission
                    {
                        key_additions.push((account_id, public_key));
                    }
                }
                StateChangeValueView::AccessKeyDeletion {
                    account_id,
                    public_key,
                } => {
                    key_deletions.push((account_id, public_key));
                }
                StateChangeValueView::AccountUpdate { .. } => {
                    // TODO: Add to vec
                }
                StateChangeValueView::AccountDeletion { .. } => {
                    // TODO: Add to vec
                }
                _ => {}
            }
        }

        StateChangeAggregator {
            key_additions,
            key_deletions,
            account_creations,
            account_deletions,
        }
    }
}

impl StateChangeAggregator {
    pub(crate) async fn execute(
        &self,
        rt: &TxProcessingRuntime,
        timestamp: NaiveDateTime,
    ) {
        // FIXME: shouldn't be transacted all at once, but 4 transactions instead.
        // FIXME: cannot insert more than 1k rows at once!
        // TODO: since this is so many transactions, retry if it fails
        // TODO: join these futures
        self.key_additions_sql(rt, timestamp).await;
        self.key_deletions_sql(rt, timestamp).await;
        self.account_creations_sql(rt, timestamp).await;
        self.account_deletions_sql(rt, timestamp).await;
    }

    async fn key_additions_sql(
        &self,
        rt: &TxProcessingRuntime,
        timestamp: NaiveDateTime,
    ) {
        if self.key_deletions.is_empty() {
            return;
        }

        let sql = format!(
            "INSERT INTO access_keys (account_id, access_key, created_at) VALUES {};",
            self.key_additions
            .iter()
            .map(|(account_id, public_key)| {
                format!("({}, {}, {})", account_id, public_key, timestamp)
            })
            .join(", ")
        );

        retry_apply_sql(rt, sql).await;
    }

    async fn key_deletions_sql(
        &self,
        rt: &TxProcessingRuntime,
        timestamp: NaiveDateTime,
    ) {
        if self.key_deletions.is_empty() {
            return;
        }

        let sql = self.key_deletions
            .iter()
            .map(|(account_id, public_key)| {
                format!(
                    "UPDATE access_keys SET removed_at = {} WHERE account_id = {} and public_key = {}",
                    timestamp,
                    account_id,
                    public_key,
                )
            })
            .join("\n");

        retry_apply_sql(rt, sql).await;
    }

    async fn account_creations_sql(
        &self,
        rt: &TxProcessingRuntime,
        _timestamp: NaiveDateTime,
    ) {
        if self.account_creations.is_empty() {
            return;
        }

        // FIXME: actual construction
        let sql = format!(
            "INSERT INTO accounts () VALUES {};",
            self.account_creations.join(", ")
        );

        retry_apply_sql(rt, sql).await;
    }

    async fn account_deletions_sql(
        &self,
        rt: &TxProcessingRuntime,
        _timestamp: NaiveDateTime,
    ) {
        if self.account_deletions.is_empty() {
            return;
        }

        // FIXME: actual construction
        let sql = format!(
            "DELETE FROM accounts WHERE {};",
            self.account_deletions.join("or")
        );

        retry_apply_sql(rt, sql).await;
    }
}

async fn retry_apply_sql(_rt: &TxProcessingRuntime, sql: String) {
    println!("{}", sql);

    // TODO: apply to DB, print error if necessary
}
