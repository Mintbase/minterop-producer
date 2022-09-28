use std::str::FromStr;

use anyhow::Result;
use hyper::{Body, Request};
use minterop_data::rpc_payloads::*;
use near_lake_framework::near_indexer_primitives::types::AccountId;
use serde::Serialize;

type Client = hyper::Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>, hyper::Body>;
#[derive(Serialize)]

pub struct Contract {
    pub contract_id: String,
}
#[derive(Serialize)]
pub struct Token {
    pub contract_id: String,
    pub token_ids: Vec<String>,
}

#[derive(Clone)]
pub(crate) struct MinteropRpcConnector {
    client: Client,
    endpoint: hyper::Uri,
}

impl MinteropRpcConnector {
    pub fn new(endpoint: &str) -> Result<Self> {
        let client = hyper::Client::builder().build(hyper_tls::HttpsConnector::new());
        let endpoint = hyper::Uri::from_str(endpoint)?;
        Ok(Self { client, endpoint })
    }

    pub async fn contract(&self, contract_id: AccountId) {
        // FIXME: using https leads to "record overflow"
        // FIXME: sometimes I need the slash, sometimes not?!
        let contract = Contract {
            contract_id: contract_id.to_string(),
        };
        let req = post_json(&self.endpoint.to_string(), &contract);

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
        // TODO: check response for status code (later)
    }

    pub async fn token(&self, contract_id: AccountId, token_ids: Vec<String>) {
        let token = Token {
            contract_id: contract_id.to_string(),
            token_ids: token_ids.clone(),
        };
        let req = post_json(
            // FIXME: using https leads to "record overflow"
            // FIXME: sometimes I need the slash, sometimes not?!
            &self.endpoint.to_string(),
            &token,
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
        // TODO: check response for status code (later)
    }
}

fn post_json<T: serde::Serialize>(uri: &str, body: &T) -> Request<Body> {
    let body = Body::from(serde_json::to_string(body).unwrap());
    Request::post(uri)
        .header("Content-Type", "application/json")
        .body(body)
        .unwrap()
}
