//! Talking to Blotter Sync over HTTP: the requests the engine makes, behind
//! [`Transport`] so its tests can run against an in-memory server.

use std::future::Future;
use std::time::Duration;

use blotter_core::sync::api::{
    ErrorBody, HEADER_PROTOCOL, HEADER_SCHEMA, Hello, PullResponse, PushRequest, PushResponse,
    Versions, path,
};
use reqwest::{Client, RequestBuilder, Response};
use serde::de::DeserializeOwned;

use super::address;

/// Why a request didn't get an answer the engine can use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncError {
    /// No answer: the server is down, the address is wrong, or this device
    /// is offline (or the server failed in a way worth retrying).
    Unreachable(String),
    /// The server answered with one of its typed refusals.
    Refused(ErrorBody),
    /// A plain `http://` address outside the LAN and Tailscale (D-S6).
    Insecure(String),
    /// An answer the engine can't read.
    Bad(String),
}

impl std::fmt::Display for SyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncError::Unreachable(m) | SyncError::Insecure(m) | SyncError::Bad(m) => {
                f.write_str(m)
            }
            SyncError::Refused(body) => f.write_str(&body.message),
        }
    }
}

/// What the engine asks of a server.
pub trait Transport: Send + Sync {
    fn hello(&self, base: &str) -> impl Future<Output = Result<Hello, SyncError>> + Send;
    fn push(
        &self,
        base: &str,
        token: &str,
        req: &PushRequest,
    ) -> impl Future<Output = Result<PushResponse, SyncError>> + Send;
    fn pull(
        &self,
        base: &str,
        token: &str,
        epoch: &str,
        since: i64,
    ) -> impl Future<Output = Result<PullResponse, SyncError>> + Send;
}

/// How long `hello` may take before the next address is tried.
const HELLO_TIMEOUT: Duration = Duration::from_secs(4);
/// Any other request; a first push can carry a whole planner.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(4);
/// The change stream sends a keep-alive every 20 s; three missed means the
/// connection is gone.
const STREAM_READ_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Clone)]
pub struct Http {
    api: Client,
    stream: Client,
}

impl Http {
    pub fn new() -> Result<Self, String> {
        let agent = format!("Blotter/{}", env!("CARGO_PKG_VERSION"));
        let api = Client::builder()
            .user_agent(&agent)
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .gzip(true)
            .build()
            .map_err(|e| e.to_string())?;
        let stream = Client::builder()
            .user_agent(&agent)
            .connect_timeout(CONNECT_TIMEOUT)
            .read_timeout(STREAM_READ_TIMEOUT)
            .build()
            .map_err(|e| e.to_string())?;
        Ok(Self { api, stream })
    }

    /// Opens the change stream (`GET /v1/events`); the caller reads it.
    pub async fn events(&self, base: &str, token: &str) -> Result<Response, SyncError> {
        let resp = gated(self.stream.get(format!("{base}{}", path::EVENTS)), token)
            .header("accept", "text/event-stream")
            .send()
            .await
            .map_err(unreachable)?;
        if resp.status().is_success() {
            Ok(resp)
        } else {
            Err(refusal(resp).await)
        }
    }
}

fn gated(req: RequestBuilder, token: &str) -> RequestBuilder {
    let req = req
        .header(HEADER_PROTOCOL, Versions::CURRENT.protocol)
        .header(HEADER_SCHEMA, Versions::CURRENT.schema);
    if token.is_empty() {
        req
    } else {
        req.bearer_auth(token)
    }
}

fn unreachable(err: reqwest::Error) -> SyncError {
    let what = if err.is_timeout() {
        "timed out"
    } else if err.is_connect() {
        "couldn't connect"
    } else {
        "the connection failed"
    };
    SyncError::Unreachable(what.into())
}

async fn refusal(resp: Response) -> SyncError {
    let status = resp.status();
    match resp.json::<ErrorBody>().await {
        Ok(body) => SyncError::Refused(body),
        Err(_) if status.is_server_error() || status.as_u16() == 404 => {
            SyncError::Unreachable(format!("the server answered {status}"))
        }
        Err(_) => SyncError::Bad(format!(
            "the server answered {status}, which Blotter doesn't understand"
        )),
    }
}

async fn read<T: DeserializeOwned>(req: RequestBuilder) -> Result<T, SyncError> {
    let resp = req.send().await.map_err(unreachable)?;
    if !resp.status().is_success() {
        return Err(refusal(resp).await);
    }
    resp.json::<T>()
        .await
        .map_err(|e| SyncError::Bad(format!("couldn't read the server's answer: {e}")))
}

impl Transport for Http {
    async fn hello(&self, base: &str) -> Result<Hello, SyncError> {
        address::check(base).await.map_err(SyncError::Insecure)?;
        read(
            self.api
                .get(format!("{base}{}", path::HELLO))
                .timeout(HELLO_TIMEOUT),
        )
        .await
    }

    async fn push(
        &self,
        base: &str,
        token: &str,
        req: &PushRequest,
    ) -> Result<PushResponse, SyncError> {
        read(gated(self.api.post(format!("{base}{}", path::PUSH)), token).json(req)).await
    }

    async fn pull(
        &self,
        base: &str,
        token: &str,
        epoch: &str,
        since: i64,
    ) -> Result<PullResponse, SyncError> {
        let mut url = reqwest::Url::parse(&format!("{base}{}", path::PULL))
            .map_err(|_| SyncError::Bad(format!("“{base}” isn't an address")))?;
        url.query_pairs_mut()
            .append_pair("epoch", epoch)
            .append_pair("since", &since.to_string());
        read(gated(self.api.get(url), token)).await
    }
}
