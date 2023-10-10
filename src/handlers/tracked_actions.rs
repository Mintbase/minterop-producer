use chrono::NaiveDateTime;
use near_lake_framework::near_indexer_primitives::{
    types::AccountId,
    views::ActionView,
    CryptoHash,
};

use crate::{
    database::ExecuteDb,
    handlers::prelude::*,
};

pub(crate) enum TrackedAction {
    AddKey(AddKey),
    DeleteKey(DeleteKey),
    CreateAccount(CreateAccount),
    DeleteAccount(DeleteAccount),
}

impl TrackedAction {
    pub fn try_new(
        account_id: &AccountId,
        timestamp: NaiveDateTime,
        receipt_id: &CryptoHash,
        view: &ActionView,
    ) -> Option<TrackedAction> {
        use near_lake_framework::near_indexer_primitives::views::AccessKeyPermissionView;

        match view {
            ActionView::AddKey {
                public_key,
                access_key,
            } => match access_key.permission {
                AccessKeyPermissionView::FullAccess => {
                    Some(TrackedAction::AddKey(AddKey {
                        account_id: account_id.to_string(),
                        public_key: public_key.to_string(),
                        timestamp,
                        receipt_id: receipt_id.to_string(),
                    }))
                }
                _ => None,
            },
            ActionView::DeleteKey { public_key } => {
                Some(TrackedAction::DeleteKey(DeleteKey {
                    account_id: account_id.to_string(),
                    public_key: public_key.to_string(),
                    timestamp,
                    receipt_id: receipt_id.to_string(),
                }))
            }
            ActionView::CreateAccount => {
                Some(TrackedAction::CreateAccount(CreateAccount {
                    account_id: account_id.to_string(),
                    timestamp,
                    receipt_id: receipt_id.to_string(),
                }))
            }
            ActionView::DeleteAccount { beneficiary_id } => {
                Some(TrackedAction::DeleteAccount(DeleteAccount {
                    account_id: account_id.to_string(),
                    timestamp,
                    receipt_id: receipt_id.to_string(),
                    beneficiary_id: beneficiary_id.to_string(),
                }))
            }
            _ => None,
        }
    }

    pub async fn process(self, rt: &crate::runtime::TxProcessingRuntime) {
        match self {
            TrackedAction::AddKey(a) => a.process(rt).await,
            TrackedAction::DeleteKey(a) => a.process(rt).await,
            TrackedAction::CreateAccount(a) => a.process(rt).await,
            TrackedAction::DeleteAccount(a) => a.process(rt).await,
        }
    }
}

pub(crate) struct AddKey {
    account_id: String,
    public_key: String,
    timestamp: NaiveDateTime,
    receipt_id: String,
}

impl AddKey {
    async fn process(self, rt: &crate::runtime::TxProcessingRuntime) {
        diesel::insert_into(access_keys::table)
            .values(AccessKey {
                account_id: self.account_id,
                public_key: self.public_key,
                created_at: self.timestamp,
                created_receipt_id: self.receipt_id.clone(),
                removed_at: None,
                removed_receipt_id: None,
            })
            .execute_db_action(
                &rt.pg_connection,
                &self.receipt_id,
                "insert new access key",
            )
            .await;
    }
}

pub(crate) struct DeleteKey {
    account_id: String,
    public_key: String,
    timestamp: NaiveDateTime,
    receipt_id: String,
}

impl DeleteKey {
    async fn process(self, rt: &crate::runtime::TxProcessingRuntime) {
        use access_keys::dsl;

        diesel::update(
            dsl::access_keys
                .filter(dsl::account_id.eq(self.account_id))
                .filter(dsl::public_key.eq(self.public_key))
                .filter(dsl::removed_at.is_not_null()),
        )
        .set((
            dsl::removed_at.eq(self.timestamp),
            dsl::removed_receipt_id.eq(self.receipt_id.clone()),
        ))
        .execute_db_action(
            &rt.pg_connection,
            &self.receipt_id,
            "mark access key as removed",
        )
        .await;
    }
}

pub(crate) struct CreateAccount {
    account_id: String,
    timestamp: NaiveDateTime,
    receipt_id: String,
}

impl CreateAccount {
    async fn process(self, rt: &crate::runtime::TxProcessingRuntime) {
        diesel::insert_into(accounts::table)
            .values(Account {
                account_id: self.account_id,
                created_at: self.timestamp,
                created_receipt_id: self.receipt_id.clone(),
                removed_at: None,
                removed_receipt_id: None,
                beneficiary_id: None,
            })
            .execute_db_action(
                &rt.pg_connection,
                &self.receipt_id,
                "insert new account",
            )
            .await;
    }
}

pub(crate) struct DeleteAccount {
    account_id: String,
    timestamp: NaiveDateTime,
    receipt_id: String,
    beneficiary_id: String,
}

impl DeleteAccount {
    async fn process(self, rt: &crate::runtime::TxProcessingRuntime) {
        use accounts::dsl;

        diesel::update(
            dsl::accounts
                .filter(dsl::account_id.eq(self.account_id))
                .filter(dsl::removed_at.is_not_null()),
        )
        .set((
            dsl::removed_at.eq(self.timestamp),
            dsl::removed_receipt_id.eq(self.receipt_id.clone()),
            dsl::beneficiary_id.eq(self.beneficiary_id),
        ))
        .execute_db_action(
            &rt.pg_connection,
            &self.receipt_id,
            "mark access key as removed",
        )
        .await;
    }
}
