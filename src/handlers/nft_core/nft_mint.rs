use mb_sdk::events::nft_core::NftMintLog;

use crate::{
    error, handlers::prelude::*, runtime::TxProcessingRuntime, ReceiptData,
};

pub(crate) async fn handle_nft_mint(
    rt: &TxProcessingRuntime,
    tx: &ReceiptData,
    data: serde_json::Value,
) {
    // contract should always be inserted prior to token for metadata resolve
    rt.minterop_rpc.contract(tx.receiver.clone(), false).await;

    match serde_json::from_value::<Vec<NftMintLog>>(data.clone()) {
        Err(_) => error!(r#"Invalid log for "nft_mint": {} ({:?})"#, data, tx),
        Ok(data_logs) => {
            future::join_all(
                data_logs.into_iter().map(|log| {
                    handle_nft_mint_log(rt.clone(), tx.clone(), log)
                }),
            )
            .await;
        }
    }
}

async fn handle_nft_mint_log(
    rt: TxProcessingRuntime,
    tx: ReceiptData,
    log: NftMintLog,
) {
    // TODO: join in RPC call? -> would require `on_conflict`
    future::join(
        insert_nft_tokens(rt.clone(), tx.clone(), log.clone()),
        insert_nft_activities(rt.clone(), tx.clone(), log.clone()),
    )
    .await;

    // Async block prevents runtime borrow from being invalidated
    #[allow(clippy::redundant_async_block)]
    actix_rt::spawn(async move {
        rt.minterop_rpc
            .token(
                tx.receiver.clone(),
                log.token_ids,
                Some(tx.sender.to_string()),
            )
            .await
    });
}

async fn insert_nft_tokens(
    rt: TxProcessingRuntime,
    tx: ReceiptData,
    log: NftMintLog,
) {
    // FIXME: only try on mintbase contracts!
    let (royalties_percent, royalties, splits) =
        if log.memo.is_some() && tx.receiver.ends_with(&rt.mintbase_root) {
            parse_mint_memo(log.memo.clone().unwrap().as_str(), &tx)
        } else {
            (None, None, None)
        };

    let tokens = log
        .token_ids
        .iter()
        .map(|token_id| NftToken {
            token_id: token_id.clone(),
            nft_contract_id: tx.receiver.to_string(),
            owner: log.owner_id.clone(),
            mint_memo: log.memo.clone(),
            minted_timestamp: Some(tx.timestamp),
            minted_receipt_id: Some(tx.id.clone()),
            minter: Some(tx.sender.to_string()),
            royalties: royalties.clone(),
            royalties_percent,
            splits: splits.clone(),
            ..NftToken::empty()
        })
        .collect::<Vec<_>>();

    diesel::insert_into(nft_tokens::table)
        .values(tokens)
        .execute_db(&rt.pg_connection, &tx, "insert token on mint")
        .await
}

async fn insert_nft_activities(
    rt: TxProcessingRuntime,
    tx: ReceiptData,
    log: NftMintLog,
) {
    let activities = log
        .token_ids
        .iter()
        .map(|token_id| NftActivity {
            receipt_id: tx.id.clone(),
            tx_sender: tx.sender.to_string(),
            sender_pk: tx.sender_pk.clone(),
            timestamp: tx.timestamp,
            nft_contract_id: tx.receiver.to_string(),
            token_id: token_id.clone(),
            kind: NFT_ACTIVITY_KIND_MINT.to_string(),
            action_sender: tx.sender.to_string(),
            action_receiver: Some(log.owner_id.clone()),
            memo: log.memo.clone(),
            price: None,
            currency: None,
        })
        .collect::<Vec<_>>();

    diesel::insert_into(nft_activities::table)
        .values(activities)
        .execute_db(&rt.pg_connection, &tx, "insert activity on mint")
        .await
}

// ----------- logic for parsing mint memos on MB token contracts ----------- //
fn parse_mint_memo(
    memo: &str,
    tx: &ReceiptData,
) -> (
    Option<i32>,
    Option<serde_json::Value>,
    Option<serde_json::Value>,
) {
    match serde_json::from_str::<mb_sdk::types::nft_core::MintbaseMintMemo>(
        memo,
    ) {
        Err(e) => {
            error!(r#"Invalid mint memo: {} ({:?})"#, e, tx);
            (None, None, None)
        }
        Ok(memo) => {
            let royalties_percent = memo
                .royalty
                .clone()
                .map(|royalty| royalty.percentage.numerator as i32);

            let royalties = memo
                .royalty
                .map(|royalty| {
                    crate::util::map_fractions_to_u16(&royalty.split_between)
                })
                .and_then(|map| serde_json::to_value(map).ok());

            let splits = memo
                .split_owners
                .map(|split_owners| {
                    crate::util::map_fractions_to_u16(
                        &split_owners.split_between,
                    )
                })
                .and_then(|map| serde_json::to_value(map).ok());

            (royalties_percent, royalties, splits)
        }
    }
}
