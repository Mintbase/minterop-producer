use crate::handlers::prelude::*;

#[derive(serde::Deserialize)]
pub struct CreateMetadataData {
    metadata_id: u64,
    creator: near_sdk::AccountId,
    minters_allowlist: Option<Vec<near_sdk::AccountId>>,
    price: near_sdk::json_types::U128,
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
            tx.receiver.to_string(),
            data.metadata_id,
            data.minters_allowlist.map(|accounts| {
                accounts.into_iter().map(|a| a.to_string()).collect()
            }),
            data.price.0,
            data.creator.to_string(),
        )
        .await;
}
