use anyhow::Result;
use near_lake_framework::LakeConfigBuilder;

use crate::{
    rpc_connection::MinteropRpcConnector,
    runtime::MintlakeRuntime,
};

#[derive(serde::Deserialize)]
pub struct Config {
    start_block_height: u64,
    stop_block_height: Option<u64>,
    postgres: String,
    s3_region_name: String,
    s3_bucket_name: String,
    rust_log: Option<String>,
    rpc_url: String,
    mintbase_root: String,
}

impl Config {
    /// Read environment variables and generate config.
    pub fn from_env() -> Result<Self> {
        Ok(envy::from_env::<Self>()?)
    }

    /// Initiates postgres connection
    pub fn get_runtime(&self) -> Result<MintlakeRuntime> {
        let minterop_rpc = MinteropRpcConnector::new(&self.rpc_url)?;
        Ok(MintlakeRuntime {
            stop_block_height: self.stop_block_height,
            pg_connection: crate::database::init_db_connection(&self.postgres),
            minterop_rpc,
            mintbase_root: self.mintbase_root.clone(),
        })
    }

    /// Initiate streaming of blocks from S3
    pub fn connect_s3(&self) -> (crate::LakeHandle, crate::LakeStreamer) {
        // FIXME: use mainnet/testnet var instead of manual region/bucket name
        let lake_config = LakeConfigBuilder::default()
            .s3_bucket_name(self.s3_bucket_name.clone())
            .s3_region_name(self.s3_region_name.clone())
            .start_block_height(self.start_block_height)
            .build()
            .unwrap();
        near_lake_framework::streamer(lake_config)
    }

    /// Initializes logging from the filters defined via `RUST_LOG`
    pub fn init_logging(&self) -> Result<()> {
        let mut env_filter = tracing_subscriber::EnvFilter::new("");

        if let Some(rust_log) = &self.rust_log {
            if !rust_log.is_empty() {
                for directive in rust_log.split(',').map(parse_directive) {
                    env_filter = env_filter.add_directive(directive?);
                }
            }
        }

        tracing_subscriber::fmt::Subscriber::builder()
            .with_env_filter(env_filter)
            .with_writer(std::io::stdout)
            .init();

        Ok(())
    }

    /// Migrates the database to the most recent schema
    pub fn migrate_db(&self) -> Result<()> {
        minterop_common::run_migrations(&self.postgres)
    }
}

fn parse_directive(s: &str) -> Result<tracing_subscriber::filter::Directive> {
    Ok(s.parse()?)
}
