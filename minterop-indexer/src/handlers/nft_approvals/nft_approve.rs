use mb_sdk::events::nft_approvals::NftApproveLog;

use crate::handlers::prelude::*;

pub(crate) async fn handle_nft_approve(
    rt: &TxProcessingRuntime,
    tx: &ReceiptData,
    data: serde_json::Value,
) {
    match serde_json::from_value::<Vec<NftApproveLog>>(data.clone()) {
        Err(_) => {
            error!(r#"Invalid log for "nft_transfer": {} ({:?})"#, data, tx)
        }
        Ok(data_logs) => {
            future::join_all(data_logs.into_iter().map(|log| {
                handle_nft_approve_log(rt.clone(), tx.clone(), log)
            }))
            .await;
        }
    }
}

async fn handle_nft_approve_log(
    rt: TxProcessingRuntime,
    tx: ReceiptData,
    log: NftApproveLog,
) {
    future::join(
        insert_nft_approvals(rt.clone(), tx.clone(), log.clone()),
        insert_nft_activities(rt.clone(), tx.clone(), log.clone()),
    )
    .await;
}

async fn insert_nft_approvals(
    rt: TxProcessingRuntime,
    tx: ReceiptData,
    log: NftApproveLog,
) {
    use minterop_common::schema::nft_approvals::dsl;

    diesel::insert_into(nft_approvals::table)
        .values(NftApproval {
            nft_contract_id: tx.receiver.to_string(),
            token_id: log.token_id,
            approved_account_id: log.account_id,
            approval_id: pg_numeric(log.approval_id),
            receipt_id: tx.id.clone(),
            timestamp: tx.timestamp,
        })
        .on_conflict(diesel::pg::upsert::on_constraint("nft_approvals_pkey"))
        .do_update()
        .set((dsl::approval_id.eq(pg_numeric(log.approval_id)),))
        .execute_db(&rt.pg_connection, &tx, "insert token on transfer")
        .await
}

async fn insert_nft_activities(
    rt: TxProcessingRuntime,
    tx: ReceiptData,
    log: NftApproveLog,
) {
    diesel::insert_into(nft_activities::table)
        .values(NftActivity {
            receipt_id: tx.id.clone(),
            tx_sender: tx.sender.to_string(),
            sender_pk: tx.sender_pk.clone(),
            timestamp: tx.timestamp,
            nft_contract_id: tx.receiver.to_string(),
            token_id: log.token_id,
            kind: NFT_ACTIVITY_KIND_APPROVE.to_string(),
            action_sender: Some(tx.sender.to_string()),
            action_receiver: Some(log.account_id.to_string()),
            memo: None,
            price: None,
        })
        .execute_db(&rt.pg_connection, &tx, "insert activity on transfer")
        .await
}
