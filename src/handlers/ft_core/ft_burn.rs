use mb_sdk::events::ft_core::FtBurnLog;

use crate::{
    error,
    handlers::prelude::*,
    runtime::TxProcessingRuntime,
    ReceiptData,
};

pub(crate) async fn handle_ft_burn(
    rt: &TxProcessingRuntime,
    tx: &ReceiptData,
    data: serde_json::Value,
) {
    // contract should always be inserted prior to token for metadata resolve
    rt.minterop_rpc.contract(tx.receiver.clone(), false).await;

    match serde_json::from_value::<Vec<FtBurnLog>>(data.clone()) {
        Err(_) => error!(r#"Invalid log for "ft_burn": {} ({:?})"#, data, tx),
        Ok(data_logs) => {
            future::join_all(
                data_logs
                    .into_iter()
                    .map(|log| handle_ft_burn_log(rt, tx, log)),
            )
            .await;
        }
    }
}

async fn handle_ft_burn_log(
    rt: &TxProcessingRuntime,
    tx: &ReceiptData,
    log: FtBurnLog,
) {
    future::join(
        insert_ft_tokens(rt.clone(), tx.clone(), log.clone()),
        insert_ft_activities(rt.clone(), tx.clone(), log.clone()),
    )
    .await;
}

async fn insert_ft_tokens(
    rt: TxProcessingRuntime,
    tx: ReceiptData,
    log: FtBurnLog,
) {
    use minterop_data::schema::ft_balances::dsl;

    let tokens = FtBalance {
            ft_contract_id: tx.receiver.to_string(),
            owner: tx.sender.to_string(),
            amount: log.amount.clone(),
        };

    diesel::insert_into(ft_tokens::table)
        .values(tokens)
        .on_conflict(diesel::pg::upsert::on_constraint("ft_tokens_pkey"))
        .do_update()
        .set((
            dsl::burned_timestamp.eq(tx.timestamp),
            dsl::burned_receipt_id.eq(tx.id.clone()),
        ))
        .execute_db(&rt.pg_connection, &tx, "insert token on transfer")
        .await
}

async fn insert_ft_activities(
    rt: TxProcessingRuntime,
    tx: ReceiptData,
    log: FtBurnLog,
) {
    let activities = FtActivity {
            receipt_id: tx.id.clone(),
            timestamp: tx.timestamp,
            ft_contract_id: tx.receiver.to_string(),
            kind: NFT_ACTIVITY_KIND_BURN.to_string(),
            action_sender: tx.sender.to_string(),
            action_receiver: None,
            memo: None,
            amount: log.amount.clone(),
        };

    diesel::insert_into(ft_activities::table)
        .values(activities)
        .execute_db(&rt.pg_connection, &tx, "insert activity on mint")
        .await
}

