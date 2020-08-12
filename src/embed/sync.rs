#![allow(clippy::redundant_pattern_matching)] // For derive(DeJson).

use super::http as embed_http;
use super::types::*;
use hyper::client::HttpConnector;
use hyper::Client;
use nanoserde::{DeJson, DeJsonErr, SerJson};
use std::fmt::Debug;

#[cfg(default)]
pub const USE_BROWSER_FETCH: bool = false;

#[cfg(not(default))]
pub const USE_BROWSER_FETCH: bool = true;

pub async fn begin_sync(
    hyper_client: Option<&Client<HttpConnector>>,
    begin_sync_req: &BeginSyncRequest,
) -> Result<BeginSyncResponse, BeginSyncError> {
    let _pull_resp = pull(hyper_client, begin_sync_req).await?;
    // TODO do something with the response
    Ok(BeginSyncResponse {})
}

#[derive(Debug)]
pub enum BeginSyncError {
    PullFailed(PullError),
}

impl From<PullError> for BeginSyncError {
    fn from(err: PullError) -> BeginSyncError {
        BeginSyncError::PullFailed(err)
    }
}

#[derive(Default, SerJson)]
pub struct PullRequest {
    #[nserde(rename = "clientViewAuth")]
    pub client_view_auth: String,
    #[nserde(rename = "clientID")]
    pub client_id: String,
    #[nserde(rename = "baseStateID")]
    pub base_state_id: String,
    #[nserde(rename = "checksum")]
    pub checksum: String,
}

#[derive(Default, DeJson)]
pub struct PullResponse {
    #[nserde(rename = "stateID")]
    #[allow(dead_code)]
    state_id: String,
    #[nserde(rename = "lastMutationID")]
    #[allow(dead_code)]
    last_mutation_id: String,
    // 	TODO Patch          []kv.Operation `json:"patch"`
    #[nserde(rename = "checksum")]
    #[allow(dead_code)]
    checksum: String,
    // TODO ClientViewInfo ClientViewInfo `json:"clientViewInfo"`
}

// client will be none when using browser
pub async fn pull(
    hyper_client: Option<&Client<HttpConnector>>,
    begin_sync_req: &BeginSyncRequest,
) -> Result<PullResponse, PullError> {
    let pull_req = PullRequest {
        client_view_auth: begin_sync_req.data_layer_auth.clone(),
        client_id: "TODO".to_string(),
        base_state_id: "TODO".to_string(),
        checksum: "TODO".to_string(),
    };
    let http_req = new_pull_http_request(
        &pull_req,
        &begin_sync_req.diff_server_url,
        &begin_sync_req.diff_server_auth,
    )?;

    let http_resp: http::Response<String>;
    if USE_BROWSER_FETCH {
        http_resp = embed_http::browser_fetch(http_req).await?;
    } else {
        let client = hyper_client.ok_or_else(|| PullError::NoHttpClient)?;
        http_resp = embed_http::rust_fetch(client, http_req).await?;
    }
    if http_resp.status() != http::StatusCode::OK {
        return Err(PullError::FetchNotOk(http_resp.status()));
    }
    let pull_resp: PullResponse = DeJson::deserialize_json(http_resp.body())?;
    // TODO do something with it
    Ok(pull_resp)
}

// Pulled into a helper fn because we use it integration tests.
pub fn new_pull_http_request(
    pull_req: &PullRequest,
    diff_server_url: &str,
    diff_server_auth: &str,
) -> Result<http::Request<String>, PullError> {
    let body = SerJson::serialize_json(pull_req);
    let builder = http::request::Builder::new();
    let http_req = builder
        .method("POST")
        .uri(diff_server_url)
        .header("Content-type", "application/json")
        .header("Authorization", diff_server_auth)
        .header("X-Replicache-SyncID", "TODO")
        .body(body)?;
    Ok(http_req)
}

#[derive(Debug)]
pub enum PullError {
    FetchFailed(embed_http::FetchError),
    FetchNotOk(http::StatusCode),
    InvalidRequest(http::Error),
    InvalidResponse(DeJsonErr),
    NoHttpClient,
}

impl From<http::Error> for PullError {
    fn from(err: http::Error) -> PullError {
        PullError::InvalidRequest(err)
    }
}

impl From<embed_http::FetchError> for PullError {
    fn from(err: embed_http::FetchError) -> PullError {
        PullError::FetchFailed(err)
    }
}

impl From<DeJsonErr> for PullError {
    fn from(err: DeJsonErr) -> PullError {
        PullError::InvalidResponse(err)
    }
}
