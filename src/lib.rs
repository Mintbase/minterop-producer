mod config;
mod database;
mod handlers;
mod logging;
mod rpc_connection;
mod runtime;

pub use config::Config;
pub use runtime::MintlakeRuntime;
pub(crate) use runtime::ReceiptData;

pub type LakeStreamer = tokio::sync::mpsc::Receiver<
    near_lake_framework::near_indexer_primitives::StreamerMessage,
>;
pub type LakeHandle = tokio::task::JoinHandle<Result<(), anyhow::Error>>;

#[macro_export]
macro_rules! forward_mod {
    ($mod:ident) => {
        mod $mod;
        pub(crate) use $mod::*;
    };
}
