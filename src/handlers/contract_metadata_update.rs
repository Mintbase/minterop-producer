use crate::handlers::prelude::*;

pub(crate) async fn handle_contract_metadata_update(
    rt: &TxProcessingRuntime,
    tx: &ReceiptData,
) {
    // TODO: refresh needs to be true
    rt.minterop_rpc.contract(tx.receiver.clone(), true).await;
}
