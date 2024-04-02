use crate::handlers::prelude::*;

#[derive(serde::Deserialize)]
pub struct CreateMetadataData {
    metadata_id: near_sdk::json_types::U64,
    creator: near_sdk::AccountId,
    minters_allowlist: Option<Vec<near_sdk::AccountId>>,
    price: near_sdk::json_types::U128,
    ft_contract_id: Option<near_sdk::AccountId>,
    royalty: Option<mb_sdk::types::nft_core::Royalty>,
    max_supply: Option<u32>,
    last_possible_mint: Option<near_sdk::json_types::U64>,
    is_locked: bool,
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
            data.metadata_id.0,
            data.minters_allowlist.map(|accounts| {
                accounts.into_iter().map(|a| a.to_string()).collect()
            }),
            data.price.0,
            data.ft_contract_id.map(|id| id.to_string()),
            data.royalty
                .as_ref()
                .map(|r| crate::util::map_fractions_to_u16(&r.split_between)),
            data.royalty.map(|r| r.percentage.numerator as u16),
            data.max_supply,
            data.last_possible_mint.map(|t| t.0),
            data.is_locked,
            data.creator.to_string(),
        )
        .await;
}
