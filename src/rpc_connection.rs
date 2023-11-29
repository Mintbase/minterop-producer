use std::str::FromStr;

use anyhow::Result;
use hyper::{Body, Request};
use minterop_data::rpc_payloads::*;
use near_lake_framework::near_indexer_primitives::types::AccountId;

type Client = hyper::Client<
    hyper_tls::HttpsConnector<hyper::client::HttpConnector>,
    hyper::Body,
>;

#[derive(Clone)]
pub(crate) struct MinteropRpcConnector {
    client: Client,
    endpoint: hyper::Uri,
}

impl MinteropRpcConnector {
    pub fn new(endpoint: &str) -> Result<Self> {
        let client =
            hyper::Client::builder().build(hyper_tls::HttpsConnector::new());
        let endpoint = hyper::Uri::from_str(endpoint)?;
        Ok(Self { client, endpoint })
    }

    pub async fn contract(&self, contract_id: AccountId, refresh: bool) {
        let req = post_json(
            &self.endpoint.to_string(),
            &RpcMessage::from_contract(contract_id.to_string(), refresh),
        );

        crate::debug!("req: {:?}", req);
        let res = self.client.request(req).await;
        crate::debug!("res: {:?}", res);

        if let Err(e) = res {
            crate::error!(
                "Failed to request contract metadata: {} ({})",
                e,
                contract_id
            )
        }
    }

    pub async fn token(
        &self,
        contract_id: AccountId,
        token_ids: Vec<String>,
        minter: Option<String>,
    ) {
        let req = post_json(
            &self.endpoint.to_string(),
            &RpcMessage::from_token(
                contract_id.to_string(),
                token_ids.clone(),
                minter,
            ),
        );

        crate::debug!("req: {:?}", req);
        let res = self.client.request(req).await;
        crate::debug!("res: {:?}", res);

        if let Err(e) = res {
            crate::error!(
                "Failed to request token metadata: {} ({}<$>{:?})",
                e,
                contract_id,
                token_ids
            )
        }
    }

    pub async fn create_metadata(
        &self,
        contract_id: String,
        metadata_id: u64,
        minters_allowlist: Option<Vec<String>>,
        price: u128,
        creator: String,
    ) {
        let req = post_json(
            &self.endpoint.to_string(),
            &RpcMessage::from_metadata(
                contract_id.clone(),
                metadata_id,
                minters_allowlist,
                price,
                creator,
            ),
        );

        crate::debug!("req: {:?}", req);
        let res = self.client.request(req).await;
        crate::debug!("res: {:?}", res);

        if let Err(e) = res {
            crate::error!(
                "Failed to request metadata creation: {} ({}<$>{})",
                e,
                contract_id,
                metadata_id
            )
        }
    }

    pub async fn sale(
        &self,
        contract_id: String,
        token_id: String,
        new_owner_id: String,
        receipt_id: String,
    ) {
        let req = post_json(
            &self.endpoint.to_string(),
            &RpcMessage::from_sale(
                contract_id.clone(),
                token_id.clone(),
                new_owner_id.clone(),
                receipt_id,
            ),
        );

        crate::debug!("req: {:?}", req);
        let res = self.client.request(req).await;
        crate::debug!("res: {:?}", res);

        if let Err(e) = res {
            crate::error!(
                "Failed to dispatch sale event: {} ({}::{} -> {})",
                e,
                contract_id,
                token_id,
                new_owner_id
            )
        }
    }
}

fn post_json<T: serde::Serialize>(uri: &str, body: &T) -> Request<Body> {
    let body = Body::from(serde_json::to_string(body).unwrap());
    Request::post(uri)
        .header("Content-Type", "application/json")
        .body(body)
        .unwrap()
}
