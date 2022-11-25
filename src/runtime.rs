use near_lake_framework::near_indexer_primitives::{
    types::AccountId,
    views::BlockHeaderView,
    IndexerExecutionOutcomeWithReceipt,
    StreamerMessage,
};

use crate::{
    database::DbConnPool,
    logging::HandleErr,
    rpc_connection::MinteropRpcConnector,
    LakeStreamer,
};

/// Holding all the data needed to handle blocks
pub struct MintlakeRuntime {
    // TODO: latest block for skip checks (later)
    pub(crate) stop_block_height: Option<u64>,
    pub(crate) pg_connection: DbConnPool,
    pub(crate) minterop_rpc: MinteropRpcConnector,
    pub(crate) mintbase_root: String,
    pub(crate) contract_filter: Option<Vec<String>>,
}

impl MintlakeRuntime {
    /// Listen to a stream of blocks, and process all the contained data.
    pub async fn handle_stream(&self, stream: LakeStreamer) {
        match (self.stop_block_height, self.contract_filter.clone()) {
            (Some(0), None) => {
                self.handle_stream_unbounded_unfiltered(stream).await
            }
            (Some(h), None) => {
                self.handle_stream_bounded_unfiltered(stream, h).await
            }
            (None, None) => {
                self.handle_stream_unbounded_unfiltered(stream).await
            }
            (Some(0), Some(filter)) => {
                self.handle_stream_unbounded_filtered(stream, &filter).await
            }
            (Some(h), Some(filter)) => {
                self.handle_stream_bounded_filtered(stream, h, &filter)
                    .await
            }
            (None, Some(filter)) => {
                self.handle_stream_unbounded_filtered(stream, &filter).await
            }
        }
    }

    /// Handles the stream of blocks until a specified height and then exits.
    /// Intended for replaying transactions.
    async fn handle_stream_bounded_unfiltered(
        &self,
        mut stream: LakeStreamer,
        stop_height: u64,
    ) {
        crate::info!("Running bounded indexer to height {}", stop_height);

        #[allow(unused_assignments)]
        let mut height = 0;
        while let Some(msg) = stream.recv().await {
            height = self.handle_msg_unfiltered(msg).await;
            if height > stop_height {
                crate::info!(
                    "Finished running indexer to height, {}",
                    stop_height
                );
                return;
            }
        }
    }

    /// Handles the stream of blocks up to infinity
    async fn handle_stream_unbounded_unfiltered(
        &self,
        mut stream: LakeStreamer,
    ) {
        crate::info!("Running unbouned indexer");

        while let Some(msg) = stream.recv().await {
            self.handle_msg_unfiltered(msg).await;
        }
    }

    /// Handles the stream of blocks until a specified height and then exits.
    /// Intended for replaying transactions.
    async fn handle_stream_bounded_filtered(
        &self,
        mut stream: LakeStreamer,
        stop_height: u64,
        filter: &[String],
    ) {
        crate::info!("Running bounded indexer to height {}", stop_height);

        #[allow(unused_assignments)]
        let mut height = 0;
        while let Some(msg) = stream.recv().await {
            height = self.handle_msg_filtered(msg, filter).await;
            if height > stop_height {
                crate::info!(
                    "Finished running indexer to height, {}",
                    stop_height
                );
                return;
            }
        }
    }

    /// Handles the stream of blocks up to infinity
    async fn handle_stream_unbounded_filtered(
        &self,
        mut stream: LakeStreamer,
        filter: &[String],
    ) {
        crate::info!("Running unbouned indexer");

        while let Some(msg) = stream.recv().await {
            self.handle_msg_filtered(msg, filter).await;
        }
    }

    /// Handles a streamer message (which is mostly synonymous to a block) by
    /// getting all transactions, filtering for only those that are successful
    /// and have logs, and then spawn tasks that process them asynchronously.
    async fn handle_msg_unfiltered(&self, msg: StreamerMessage) -> u64 {
        let height = msg.block.header.height;

        if height % 10 == 0 {
            crate::info!("Processing block {}", height);
        }

        // async execution of all transactions in a block
        let handles = msg
            .shards
            .into_iter()
            .filter(|shard| shard.chunk.is_some())
            .flat_map(|shard| shard.receipt_execution_outcomes)
            .filter_map(|tx| filter_and_split_receipt(&msg.block.header, tx))
            .map(|(tx, logs)| {
                // This clone internally clones an Arc, and thus doesn't
                // establish a new connection on every transaction. That's what
                // we want here
                let rt = self.tx_processing_runtime();
                actix_rt::spawn(async move { handle_tx(&rt, tx, logs).await })
            })
            .collect::<Vec<_>>();

        // make sure that everything processed fine
        for handle in handles {
            handle.await.handle_err(|e| {
                crate::error!(
                    "Could not join async handle at block height {}: {:?}",
                    height,
                    e
                )
            });
        }

        update_db_blockheight(&self.pg_connection, height).await;
        height
    }

    /// The same as `handle_msg_unfiltered, but applies `
    async fn handle_msg_filtered(
        &self,
        msg: StreamerMessage,
        filter: &[String],
    ) -> u64 {
        let height = msg.block.header.height;

        if height % 10 == 0 {
            crate::info!("Processing block {}", height);
        }

        // async execution of all transactions in a block
        let handles = msg
            .shards
            .into_iter()
            .filter(|shard| shard.chunk.is_some())
            .flat_map(|shard| shard.receipt_execution_outcomes)
            .filter_map(|tx| filter_and_split_receipt(&msg.block.header, tx))
            .filter_map(|(tx, logs)| {
                if filter.contains(&tx.receiver.to_string()) {
                    Some((tx, logs))
                } else {
                    None
                }
            })
            .map(|(tx, logs)| {
                // This clone internally clones an Arc, and thus doesn't
                // establish a new connection on every transaction. That's what
                // we want here
                let rt = self.tx_processing_runtime();
                actix_rt::spawn(async move { handle_tx(&rt, tx, logs).await })
            })
            .collect::<Vec<_>>();

        // make sure that everything processed fine
        for handle in handles {
            handle.await.handle_err(|e| {
                crate::error!(
                    "Could not join async handle at block height {}: {:?}",
                    height,
                    e
                )
            });
        }

        update_db_blockheight(&self.pg_connection, height).await;
        height
    }

    fn tx_processing_runtime(&self) -> TxProcessingRuntime {
        TxProcessingRuntime {
            pg_connection: self.pg_connection.clone(),
            minterop_rpc: self.minterop_rpc.clone(),
            mintbase_root: self.mintbase_root.clone(),
        }
    }
}

/// Handles a transaction by filtering all logs for being an event log and
/// processing those in order.
async fn handle_tx(
    rt: &TxProcessingRuntime,
    tx: ReceiptData,
    logs: Vec<String>,
) {
    for log in logs
        .into_iter()
        .filter(|log| log.starts_with("EVENT_JSON:"))
    {
        handle_log(rt, tx.clone(), log).await;
    }
}

/// Parses standard, version, and event type out of an event logs, selects an
/// appropriate handler function, and passes the data.
async fn handle_log(rt: &TxProcessingRuntime, tx: ReceiptData, log: String) {
    use crate::handlers::*;

    let (standard, version, event, data) =
        match near_events::partial_deserialize_event(log.as_str()) {
            None => {
                crate::error!("Got malformed event log: {} ({:?})", log, tx);
                return;
            }
            Some(event) => sanitize_event(event),
        };

    match (standard.as_str(), version.as_str(), event.as_str()) {
        // ------------ nft_core
        ("nep171", "1.0.0", "nft_mint") => handle_nft_mint(rt, &tx, data).await,
        ("nep171", "1.0.0", "nft_transfer") => {
            handle_nft_transfer(rt, &tx, data).await
        }
        ("nep171", "1.0.0", "nft_burn") => handle_nft_burn(rt, &tx, data).await,
        // ------------ nft_approvals
        ("mb_store", "0.1.0", "nft_approve") => {
            handle_nft_approve(rt, &tx, data).await
        }
        ("mb_store", "0.1.0", "nft_revoke") => {
            handle_nft_revoke(rt, &tx, data).await
        }
        ("mb_store", "0.1.0", "nft_revoke_all") => {
            handle_nft_revoke_all(rt, &tx, data).await
        }
        // ------------ nft_payouts
        ("mb_store", "0.1.0", "nft_set_split_owners") => {
            handle_nft_set_split_owners(rt, &tx, data).await
        }
        // ------------ mb_store_settings
        ("mb_store", "0.1.0", "deploy") => {
            handle_mb_store_deploy(rt, &tx, data).await
        }
        ("mb_store", "0.1.0", "change_setting") => {
            handle_mb_store_change_setting(rt, &tx, data).await
        }
        // ------------ old mintbase market
        ("mb_market", "0.1.0", "nft_list") => {
            market_v01::handle_nft_list(rt, &tx, data).await
        }
        ("mb_market", "0.1.0", "nft_unlist") => {
            market_v01::handle_nft_unlist(rt, &tx, data).await
        }
        ("mb_market", "0.1.0", "nft_update_list") => {
            market_v01::handle_nft_update_list(rt, &tx, data).await
        }
        ("mb_market", "0.1.0", "nft_sold") => {
            market_v01::handle_nft_sold(rt, &tx, data).await
        }
        ("mb_market", "0.1.0", "nft_make_offer") => {
            market_v01::handle_nft_make_offer(rt, &tx, data).await
        }
        ("mb_market", "0.1.0", "nft_withdraw_offer") => {
            market_v01::handle_nft_withdraw_offer(rt, &tx, data).await
        }
        // ("mb_market", "0.1.0", "update_banlist") => { not necessary }
        // ("mb_market", "0.1.0", "update_allowlist") => { not necessary }
        // ------------ interop mintbase market
        ("mb_market", "0.2.1", "nft_list") => {
            market_v02::handle_nft_list(rt, &tx, data).await
        }
        ("mb_market", "0.2.1", "nft_unlist") => {
            market_v02::handle_nft_unlist(rt, &tx, data).await
        }
        // 0.2.2 extends 0.2.1 by optional field -> backwards compatible
        ("mb_market", "0.2.1", "nft_sale")
        | ("mb_market", "0.2.2", "nft_sale") => {
            market_v02::handle_nft_sold(rt, &tx, data).await
        }
        ("mb_market", "0.2.1", "nft_make_offer") => {
            market_v02::handle_nft_make_offer(rt, &tx, data).await
        }
        // only needed for auctions -> deferred
        // ("mb_market", "0.2.0", "nft_withdraw_offer") => {
        //     market_v02::handle_nft_withdraw_offer(rt, &tx, data).await
        // }
        _ => { /* not standardized, not mintbase, not interesting */ }
    }
}

#[derive(Clone)]
pub(crate) struct TxProcessingRuntime {
    pub(crate) pg_connection: DbConnPool,
    pub(crate) minterop_rpc: MinteropRpcConnector,
    pub(crate) mintbase_root: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ReceiptData {
    pub(crate) id: String,
    pub(crate) sender: AccountId,
    pub(crate) sender_pk: Option<String>,
    pub(crate) receiver: AccountId,
    pub(crate) timestamp: chrono::NaiveDateTime,
    // pub(crate) block_height: u64,
}

fn filter_and_split_receipt(
    header: &BlockHeaderView,
    tx: IndexerExecutionOutcomeWithReceipt,
) -> Option<(ReceiptData, Vec<String>)> {
    use near_lake_framework::near_indexer_primitives::views;

    // check for tx success
    match tx.execution_outcome.outcome.status {
        views::ExecutionStatusView::Unknown => None,
        views::ExecutionStatusView::Failure(_) => None,
        // check if we have any logs
        _ => match tx.execution_outcome.outcome.logs.len() {
            0 => None,
            _ => Some((
                ReceiptData {
                    id: tx.receipt.receipt_id.to_string(),
                    sender: tx.receipt.predecessor_id,
                    sender_pk: match tx.receipt.receipt {
                        views::ReceiptEnumView::Action {
                            signer_public_key,
                            ..
                        } => Some(signer_public_key.to_string()),
                        _ => None,
                    },
                    receiver: tx.receipt.receiver_id,
                    timestamp: crate::nsecs_to_timestamp(
                        header.timestamp_nanosec,
                    ),
                    // block_height: header.height,
                },
                tx.execution_outcome.outcome.logs,
            )),
        },
    }
}

/// The database should always know the last synced block, to forward to
/// frontend for quick health checks, and in perspective to get the starting
/// block height from the database. This function handles that insert.
async fn update_db_blockheight(db: &DbConnPool, height: u64) {
    use actix_diesel::dsl::AsyncRunQueryDsl;
    use diesel::ExpressionMethods;
    use minterop_data::schema::blocks::dsl::*;

    diesel::update(blocks)
        .set(synced_height.eq(height as i64))
        .execute_async(db)
        .await
        .handle_err(|e| {
            crate::error!(
                r#"Failed to set "blocks.synced_height" to {}: {}"#,
                height,
                e
            )
        });
}

fn sanitize_event(
    event: (String, String, String, serde_json::Value),
) -> (String, String, String, serde_json::Value) {
    let (nep, version, event, data) = event;

    let version = version.trim_start_matches("nft-").to_string();

    (nep, version, event, data)
}
