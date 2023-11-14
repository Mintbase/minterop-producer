use crate::handlers::prelude::*;

#[derive(serde::Deserialize)]
pub struct CreateMetadataData {
    metadata_id: u64,
    creator: near_sdk::AccountId,
}

pub(crate) async fn handle_create_metadata(
    rt: &TxProcessingRuntime,
    tx: &ReceiptData,
    data: serde_json::Value,
) {
    let data = match serde_json::from_value::<CreateMetadataData>(data.clone())
    {
        Err(_) => {
            error!(r#"Invalid log for "create_metadata": {} ({:?})"#, data, tx);
            return;
        }
        Ok(data) => data,
    };

    rt.minterop_rpc
        .create_metadata(
            tx.receiver.clone(),
            data.metadata_id,
            data.creator.to_string(),
        )
        .await;
}
