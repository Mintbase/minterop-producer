// use futures::StreamExt;
use minterop_indexer::{
    Config,
    LakeHandle,
    LakeStreamer,
    MintlakeRuntime,
};
// use tokio_stream::wrappers::ReceiverStream;

async fn init() -> (LakeHandle, LakeStreamer, MintlakeRuntime) {
    if let Err(e) = dotenv::dotenv() {
        panic!("Failed to execute `dotenv::dotenv`: {:?}", e);
    }

    // get config
    let cfg = match Config::from_env() {
        Err(e) => panic!("Failed to load config from environment: {:?}", e),
        Ok(cfg) => cfg,
    };

    // initialize all the logging
    if let Err(e) = cfg.init_logging() {
        panic!("Failed to initialize logging: {:?}", e);
    }

    // embedded migrations
    if let Err(e) = cfg.migrate_db() {
        panic!("Failed to migrate database: {:?}", e);
    }

    // database connection
    let rt = match cfg.get_runtime() {
        Err(e) => panic!("Failed to initialize runtime: {:?}", e),
        Ok(rt) => rt,
    };

    // S3 connection needs to be last to prevent buffer overflows
    let (handle, streamer) = cfg.connect_s3();

    (handle, streamer, rt)
}

#[actix_rt::main]
async fn main() {
    let (handle, streamer, rt) = init().await;
    minterop_indexer::info!("Connected, migrated, and ready to index!");
    rt.handle_stream(streamer).await;
    match handle.await {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            minterop_indexer::error!("Failed to join lake handle: {:?}", e)
        }
        Err(e) => {
            minterop_indexer::error!("Failed to join lake handle: {:?}", e)
        }
    }
}
