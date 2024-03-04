use crate::handlers::prelude::*;

#[derive(serde::Deserialize)]
struct NftMetadataUpdateLog {
    token_ids: Vec<String>,
}

#[derive(serde::Deserialize)]
pub struct NftMetadataUpdateData(Vec<NftMetadataUpdateLog>);

pub(crate) async fn handle_nft_metadata_update(
    rt: &TxProcessingRuntime,
    tx: &ReceiptData,
    data: serde_json::Value,
) {
    let data =
        match serde_json::from_value::<NftMetadataUpdateData>(data.clone()) {
            Err(_) => {
                error!(
                    r#"Invalid log for "nft_metadata_update": {} ({:?})"#,
                    data, tx
                );
                return;
            }
            Ok(data) => data,
        };

    for log in data.0 {
        rt.minterop_rpc
            .token(tx.receiver.to_string(), log.token_ids, None, Some(true))
            .await;
    }
}
