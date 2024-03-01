use mb_sdk::events::mb_store_settings::{
    MbStoreChangeSettingData, MbStoreDeployData,
};

use crate::handlers::prelude::*;

pub(crate) async fn handle_mb_store_deploy(
    rt: &TxProcessingRuntime,
    tx: &ReceiptData,
    data: serde_json::Value,
) {
    let data = match serde_json::from_value::<MbStoreDeployData>(data.clone()) {
        Err(_) => {
            error!(r#"Invalid log for "nft_transfer": {} ({:?})"#, data, tx);
            return;
        }
        Ok(data) => data,
    };

    future::join(
        // update owner and set mintbase contract
        diesel::insert_into(nft_contracts::table)
            .values(NftContract {
                id: data.store_id.clone(),
                spec: data.contract_metadata.spec,
                name: data.contract_metadata.name,
                symbol: data.contract_metadata.symbol,
                icon: data.contract_metadata.icon,
                base_uri: data.contract_metadata.base_uri,
                reference: data.contract_metadata.reference,
                reference_hash: data
                    .contract_metadata
                    .reference_hash
                    .map(|hash| serde_json::to_string(&hash).unwrap()),
                created_at: Some(tx.timestamp),
                created_receipt_id: Some(tx.id.clone()),
                owner_id: Some(data.owner_id.clone()),
                is_mintbase: true,
                category: None,
            })
            .execute_db(&rt.pg_connection, tx, "Updating new contract"),
        // add owner as minter
        diesel::insert_into(mb_store_minters::table)
            .values(MbStoreMinter {
                nft_contract_id: data.store_id.clone(),
                minter_id: data.owner_id.clone(),
                receipt_id: tx.id.clone(),
                timestamp: tx.timestamp,
            })
            .execute_db(&rt.pg_connection, tx, "insert new contract"),
    )
    .await;
}

pub(crate) async fn handle_mb_store_change_setting(
    rt: &TxProcessingRuntime,
    tx: &ReceiptData,
    data: serde_json::Value,
) {
    let data = match serde_json::from_value::<MbStoreChangeSettingData>(
        data.clone(),
    ) {
        Err(_) => {
            error!(r#"Invalid log for "nft_transfer": {} ({:?})"#, data, tx);
            return;
        }
        Ok(data) => data,
    };

    // TODO: do we really need this call?
    // call RPC for contract metadata, needs to await to avoid invalid mutation
    rt.minterop_rpc
        .contract(tx.receiver.to_string(), false)
        .await;

    if let Some(new_minter) = data.granted_minter {
        diesel::insert_into(mb_store_minters::table)
            .values(MbStoreMinter {
                nft_contract_id: tx.receiver.to_string(),
                minter_id: new_minter,
                receipt_id: tx.id.clone(),
                timestamp: tx.timestamp,
            })
            .execute_db(&rt.pg_connection, tx, "insert minter")
            .await;
    }

    if let Some(revoked_minter) = data.revoked_minter {
        diesel::delete(
            mb_store_minters::table
                .filter(
                    mb_store_minters::dsl::nft_contract_id
                        .eq(tx.receiver.to_string()),
                )
                .filter(mb_store_minters::dsl::minter_id.eq(revoked_minter)),
        )
        .execute_db(&rt.pg_connection, tx, "delete minter")
        .await;
    }

    if let Some(new_owner) = data.new_owner {
        diesel::update(
            nft_contracts::table
                .filter(nft_contracts::dsl::id.eq(tx.receiver.to_string())),
        )
        .set(nft_contracts::dsl::owner_id.eq(new_owner))
        .execute_db(&rt.pg_connection, tx, "updating owner")
        .await
    }

    if let Some(new_icon) = data.new_icon_base64 {
        diesel::update(
            nft_contracts::table
                .filter(nft_contracts::dsl::id.eq(tx.receiver.to_string())),
        )
        .set(nft_contracts::dsl::icon.eq(new_icon))
        .execute_db(&rt.pg_connection, tx, "updating owner")
        .await
    }

    if let Some(new_uri) = data.new_base_uri {
        diesel::update(
            nft_contracts::table
                .filter(nft_contracts::dsl::id.eq(tx.receiver.to_string())),
        )
        .set(nft_contracts::dsl::base_uri.eq(new_uri))
        .execute_db(&rt.pg_connection, tx, "updating owner")
        .await
    }
}
