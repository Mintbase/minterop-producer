use std::str::FromStr;

use anyhow::Result;
use hyper::{Body, Request};
use minterop_data::rpc_payloads::*;

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

    pub async fn contract(&self, contract_id: String, refresh: bool) {
        let req = post_json(
            &self.endpoint.to_string(),
            &RpcMessage::HandleContractPayload {
                contract_id: contract_id.clone(),
                refresh: Some(refresh),
            },
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
        contract_id: String,
        token_ids: Vec<String>,
        minter: Option<String>,
        refresh: Option<bool>,
    ) {
        let req = post_json(
            &self.endpoint.to_string(),
            &RpcMessage::HandleTokenPayload {
                contract_id: contract_id.clone(),
                token_ids: token_ids.clone(),
                minter,
                refresh,
            },
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

    #[allow(clippy::too_many_arguments)] // Forgive me father for I have sinned
    pub async fn create_metadata(
        &self,
        contract_id: String,
        metadata_id: u64,
        minters_allowlist: Option<Vec<String>>,
        price: u128,
        royalties: Option<crate::util::U16Map>,
        royalty_percent: Option<u16>,
        max_supply: Option<u32>,
        last_possible_mint: Option<u64>,
        is_locked: bool,
        creator: String,
    ) {
        let req = post_json(
            &self.endpoint.to_string(),
            &RpcMessage::HandleMetadataPayload {
                contract_id: contract_id.clone(),
                metadata_id,
                minters_allowlist,
                price: price.to_string(),
                royalties,
                royalty_percent,
                max_supply,
                last_possible_mint,
                is_locked,
                refresh: None,
                creator,
            },
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
            &RpcMessage::HandleSalePayload {
                contract_id: contract_id.clone(),
                token_id: token_id.clone(),
                new_owner_id: new_owner_id.clone(),
                receipt_id,
            },
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
