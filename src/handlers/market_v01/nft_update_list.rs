use mb_sdk::events::market_v1::NftUpdateListData;

use crate::handlers::prelude::*;

pub(crate) async fn handle_nft_update_list(
    rt: &TxProcessingRuntime,
    tx: &ReceiptData,
    data: serde_json::Value,
) {
    use nft_listings::dsl;

    let data = match serde_json::from_value::<NftUpdateListData>(data.clone()) {
        Err(_) => {
            error!(r#"Invalid log for "nft_update_list": {} ({:?})"#, data, tx);
            return;
        }
        Ok(data) => data,
    };

    let (nft_contract, token_id, approval_id) =
        match super::parse_list_id(&data.list_id) {
            None => {
                crate::error!(
                    "Unparseable list ID: {}, ({:?})",
                    data.list_id,
                    tx
                );
                return;
            }
            Some(triple) => triple,
        };

    let target_row = diesel::update(
        dsl::nft_listings
            .filter(dsl::token_id.eq(token_id.to_string()))
            .filter(dsl::nft_contract_id.eq(nft_contract.to_string()))
            .filter(dsl::approval_id.eq(pg_numeric(approval_id))),
    );

    if let Some(new_price) = data.price.map(|price| pg_numeric(price.0)) {
        target_row
            .set(dsl::price.eq(new_price))
            .execute_db(&rt.pg_connection, tx, "update listing")
            .await;
    } else if let Some(true) = data.auto_transfer {
        target_row
            .set(dsl::kind.eq(NFT_LISTING_KIND_SIMPLE))
            .execute_db(&rt.pg_connection, tx, "update listing")
            .await;
    } else if let Some(false) = data.auto_transfer {
        target_row
            .set(dsl::kind.eq(NFT_LISTING_KIND_AUCTION))
            .execute_db(&rt.pg_connection, tx, "update listing")
            .await;
    } else {
        crate::error!("Invalid listing update date: {:?} ({:?})", data, tx);
    }
}
