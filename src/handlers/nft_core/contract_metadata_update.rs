use crate::handlers::prelude::*;

pub(crate) async fn handle_contract_metadata_update(
    rt: &TxProcessingRuntime,
    tx: &ReceiptData,
) {
    rt.minterop_rpc
        .contract(tx.receiver.to_string(), true)
        .await;
}
