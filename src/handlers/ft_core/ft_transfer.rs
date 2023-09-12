use mb_sdk::events::ft_core::FtTransferLog;

use crate::{
    error,
    handlers::prelude::*,
    runtime::TxProcessingRuntime,
    ReceiptData,
};

pub(crate) async fn handle_ft_transfer(
    rt: &TxProcessingRuntime,
    tx: &ReceiptData,
    data: serde_json::Value,
) {
    // contract should always be inserted prior to token for metadata resolve
    rt.minterop_rpc.contract(tx.receiver.clone(), false).await;

    match serde_json::from_value::<Vec<FtTransferLog>>(data.clone()) {
        Err(_) => {
            error!(r#"Invalid log for "ft_transfer": {} ({:?})"#, data, tx)
        }
        Ok(data_logs) => {
            future::join_all(data_logs.into_iter().map(|log| {
                handle_ft_transfer_log(rt.clone(), tx.clone(), log)
            }))
            .await;
        }
    }
}

async fn handle_ft_transfer_log(
    rt: TxProcessingRuntime,
    tx: ReceiptData,
    log: FtTransferLog,
) {
    // TODO: join in RPC call? -> would require `on_conflict`
    future::join(
        insert_ft_tokens(rt.clone(), tx.clone(), log.clone()),
        insert_ft_activities(rt.clone(), tx.clone(), log.clone())
    )
    .await;

    // Async block prevents runtime borrow from being invalidated
    #[allow(clippy::redundant_async_block)]
    actix_rt::spawn(async move {
        rt.minterop_rpc
            .token(
                tx.receiver.clone(),
                log.token_ids,
                Some(tx.sender.to_string()),
            )
            .await
    });
}

async fn insert_ft_tokens(
    rt: TxProcessingRuntime,
    tx: ReceiptData,
    log: FtTransferLog,
) {
    use minterop_data::schema::ft_balances::dsl;

    let tokens = FtBalance {
            ft_contract_id: tx.receiver.to_string(),
            owner: log.new_owner_id.clone(),
            amount: log.amount.clone(),
        };

    diesel::insert_into(ft_balances::table)
        .values(tokens)
        .on_conflict(diesel::pg::upsert::on_constraint("ft_tokens_pkey"))
        .do_update()
        .set((
            dsl::owner.eq(log.new_owner_id.clone()),
        ))
        .execute_db(&rt.pg_connection, &tx, "insert tokens on transfer")
        .await
}

async fn insert_ft_activities(
    rt: TxProcessingRuntime,
    tx: ReceiptData,
    log: FtTransferLog,
) {
    let activities = FtActivity {
        receipt_id: tx.id.clone(),
        timestamp: tx.timestamp,
        ft_contract_id: tx.receiver.to_string(),
        kind: NFT_ACTIVITY_KIND_TRANSFER.to_string(),
        action_sender: log.old_owner_id.clone(),
        action_receiver: Some(log.new_owner_id.clone()),
        memo: None,
        amount: log.amount.clone(),
    };


    diesel::insert_into(ft_activities::table)
        .values(activities)
        .execute_db(&rt.pg_connection, &tx, "insert activity on transfer")
        .await
}
