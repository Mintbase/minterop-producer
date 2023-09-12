use mb_sdk::events::ft_core::FtMintLog;

use crate::{
    error,
    handlers::prelude::*,
    runtime::TxProcessingRuntime,
    ReceiptData,
};

pub(crate) async fn handle_ft_mint(
    rt: &TxProcessingRuntime,
    tx: &ReceiptData,
    data: serde_json::Value,
) {
    rt.minterop_rpc.contract(tx.receiver.clone(), false).await;

    match serde_json::from_value::<Vec<FtMintLog>>(data.clone()) {
        Err(_) => error!(r#"Invalid log for "ft_mint": {} ({:?})"#, data, tx),
        Ok(data_logs) => {
            future::join_all(
                data_logs
                    .into_iter()
                    .map(|log| handle_ft_mint_log(rt.clone(), tx.clone(), log)),
            )
            .await;
        }
    }
}

async fn handle_ft_mint_log(
    rt: TxProcessingRuntime,
    tx: ReceiptData,
    log: FtMintLog,
) {
    future::join(
        insert_ft_tokens(rt.clone(), tx.clone(), log.clone()),
        insert_ft_activities(rt.clone(), tx.clone(), log.clone()),
    )
    .await;

    // TODO: RPC call for FT metadata
}

async fn insert_ft_tokens(
    rt: TxProcessingRuntime,
    tx: ReceiptData,
    log: FtMintLog,
) {
    let tokens = FtBalance {
        ft_contract_id: tx.receiver.to_string(),
        owner: log.owner_id.clone(),
        amount: pg_numeric(log.amount.0),
    };

    // TODO: handle update case
    diesel::insert_into(ft_balances::table)
        .values(tokens)
        .execute_db(&rt.pg_connection, &tx, "insert token on mint")
        .await
}

async fn insert_ft_activities(
    rt: TxProcessingRuntime,
    tx: ReceiptData,
    log: FtMintLog,
) {
    let activities = FtActivity {
        receipt_id: tx.id.clone(),
        timestamp: tx.timestamp,
        ft_contract_id: tx.receiver.to_string(),
        kind: NFT_ACTIVITY_KIND_MINT.to_string(),
        action_sender: tx.sender.to_string(),
        action_receiver: Some(log.owner_id.clone()),
        memo: log.memo.clone(),
        amount: pg_numeric(log.amount.0),
    };

    diesel::insert_into(ft_activities::table)
        .values(activities)
        .execute_db(&rt.pg_connection, &tx, "insert activity on mint")
        .await
}
