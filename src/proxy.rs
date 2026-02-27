use std::{path::PathBuf, str::FromStr};

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::Response,
};
use http::{HeaderName, HeaderValue};
use regex::bytes::Regex;
use reqwest_middleware::ClientWithMiddleware;
use tokio::process::Command;
use urlencoding::decode;

use crate::{
    config::{CommandProxieConfig, LocalProxieConfig, ProxieConfig, RemoteProxieConfig},
    state::AppState,
};

async fn remote_proxie_handler(
    path: &str,
    proxie_cfg: &RemoteProxieConfig,
    client: &ClientWithMiddleware,
) -> Result<Response, (StatusCode, String)> {
    let url = format!("{}/{}", proxie_cfg.target.trim_end_matches('/'), path);

    let mut req = client.get(url);
    for (k, v) in &proxie_cfg.upstream_headers {
        req = req.header(k, v);
    }

    let resp = req
        .send()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Upstream error: {}", e)))?;

    let mut downstream_headers = HeaderMap::new();

    if let Some(ct) = resp.headers().get("content-type") {
        downstream_headers.insert("content-type", ct.clone());
    }

    for (k, v) in &proxie_cfg.downstream_headers {
        if let (Ok(name), Ok(val)) = (HeaderName::from_str(k), HeaderValue::from_str(v)) {
            downstream_headers.insert(name, val);
        }
    }

    let mut body = resp
        .bytes()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Body read error: {}", e)))?
        .to_vec();

    for (pattern, rep) in &proxie_cfg.rewrite {
        let re = match Regex::new(pattern) {
            Ok(r) => r,
            Err(_) => continue,
        };

        body = re.replace_all(&body, rep.as_bytes()).into_owned();
    }

    let mut response = Response::new(body.into());
    *response.headers_mut() = downstream_headers;

    return Ok(response);
}

async fn local_proxie_handler(
    path: &str,
    proxie_cfg: &LocalProxieConfig,
) -> Result<Response, (StatusCode, String)> {
    let mut target_path = PathBuf::from(&proxie_cfg.target);
    target_path.push(path.trim_start_matches('/'));

    if !target_path.is_file() {
        return Err((
            StatusCode::NOT_FOUND,
            "The requested resource is not a file or does not exist".into(),
        ));
    }

    let mut body = std::fs::read(&target_path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to read file: {}", e),
        )
    })?;

    for (pattern, rep) in &proxie_cfg.rewrite {
        let re = match Regex::new(pattern) {
            Ok(r) => r,
            Err(_) => continue,
        };

        body = re.replace_all(&body, rep.as_bytes()).into_owned();
    }

    let mut downstream_headers = HeaderMap::new();

    for (k, v) in &proxie_cfg.downstream_headers {
        if let (Ok(name), Ok(val)) = (HeaderName::from_str(k), HeaderValue::from_str(v)) {
            downstream_headers.insert(name, val);
        }
    }

    let mut response = Response::new(body.into());
    *response.headers_mut() = downstream_headers;

    return Ok(response);
}

async fn command_proxie_handler(
    path: &str,
    proxie_cfg: &CommandProxieConfig,
) -> Result<Response, (StatusCode, String)> {
    let mut command = Command::new(&proxie_cfg.target);
    command.args(&proxie_cfg.args);

    let param = decode(path)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
        .into_owned();

    command.args(param.split_whitespace());

    let output = command
        .output()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if !output.status.success() {
        return Err((
            StatusCode::BAD_REQUEST,
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    let mut body = output.stdout;

    for (pattern, rep) in &proxie_cfg.rewrite {
        let re = match Regex::new(pattern) {
            Ok(r) => r,
            Err(_) => continue,
        };

        body = re.replace_all(&body, rep.as_bytes()).into_owned();
    }

    let mut downstream_headers = HeaderMap::new();

    for (k, v) in &proxie_cfg.downstream_headers {
        if let (Ok(name), Ok(val)) = (HeaderName::from_str(k), HeaderValue::from_str(v)) {
            downstream_headers.insert(name, val);
        }
    }

    let mut response = Response::new(body.into());
    *response.headers_mut() = downstream_headers;

    return Ok(response);
}

pub async fn handler(
    Path((id, path)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Response, (StatusCode, String)> {
    let proxie_cfg = state.config.proxies.get(&id).ok_or((
        StatusCode::NOT_FOUND,
        format!("Proxie ID '{}' not found", &id),
    ))?;

    match proxie_cfg {
        ProxieConfig::Remote(remote) => remote_proxie_handler(&path, &remote, &state.client).await,
        ProxieConfig::Local(local) => local_proxie_handler(&path, &local).await,
        ProxieConfig::Command(command) => command_proxie_handler(&path, &command).await,
    }
}
