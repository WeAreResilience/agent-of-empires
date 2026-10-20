//! `aoe mcp config`: the printed client entry reaches the daemon's `/mcp` server.

use reqwest::{Client, StatusCode};
use serde_json::{json, Value};
use serial_test::parallel;

use crate::harness::TuiTestHarness;

fn block_on<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Runtime::new()
        .expect("tokio runtime")
        .block_on(f)
}

/// POST one JSON-RPC message to `url` with `headers`; returns the status and the parsed body.
async fn rpc(url: &str, headers: &[(String, String)], body: Value) -> (StatusCode, Value) {
    let mut req = Client::new()
        .post(url)
        .header("accept", "application/json, text/event-stream")
        .json(&body);
    for (name, value) in headers {
        req = req.header(name, value);
    }
    let resp = req
        .send()
        .await
        .unwrap_or_else(|e| panic!("POST {url}: {e}"));
    let status = resp.status();
    (status, resp.json().await.unwrap_or(Value::Null))
}

fn initialize() -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "clientInfo": { "name": "e2e", "version": "1" }
        }
    })
}

#[test]
#[parallel]
fn cli_mcp_config_entry_connects_to_the_daemon() {
    let mut h = TuiTestHarness::new("mcp_config_connects");
    h.append_config("[web]\nmcp_enabled = true");
    h.stop_daemon_on_drop();
    let port = h.start_daemon_with(&[]);

    let out = h.run_cli(&["mcp", "config"]);
    assert!(out.status.success(), "{out:?}");
    assert!(
        out.stderr.is_empty(),
        "no warning with the setting on: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let printed: Value = serde_json::from_slice(&out.stdout).expect("stdout is JSON");
    let entry = &printed["mcpServers"]["aoe"];
    assert_eq!(entry["type"], "http");
    let url = entry["url"].as_str().expect("url string");
    assert!(url.ends_with(&format!(":{port}/mcp")), "{url}");
    let headers: Vec<(String, String)> = entry["headers"]
        .as_object()
        .expect("token-auth daemon yields headers")
        .iter()
        .map(|(k, v)| (k.clone(), v.as_str().expect("header string").to_string()))
        .collect();

    block_on(async {
        let (status, _) = rpc(url, &[], initialize()).await;
        assert_eq!(
            status,
            StatusCode::UNAUTHORIZED,
            "the header is what authorizes"
        );

        let (status, body) = rpc(url, &headers, initialize()).await;
        assert_eq!(status, StatusCode::OK, "{body}");
        assert_eq!(body["result"]["serverInfo"]["name"], "aoe", "{body}");

        let (status, body) = rpc(
            url,
            &headers,
            json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" }),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{body}");
        let names: Vec<&str> = body["result"]["tools"]
            .as_array()
            .expect("tools array")
            .iter()
            .filter_map(|t| t["name"].as_str())
            .collect();
        assert!(names.contains(&"list_sessions"), "{names:?}");
    });
}

#[test]
#[parallel]
fn cli_mcp_config_guides_when_no_daemon_or_setting_is_off() {
    let mut h = TuiTestHarness::new("mcp_config_guidance");
    let stderr = h.run_cli_err(&["mcp", "config"]);
    assert!(stderr.contains("aoe serve --daemon"), "{stderr}");

    h.stop_daemon_on_drop();
    let port = h.start_daemon();
    let out = h.run_cli(&["mcp", "config"]);
    assert!(out.status.success(), "{out:?}");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("mcp_enabled"), "{stderr}");
    let printed: Value = serde_json::from_slice(&out.stdout).expect("stdout is JSON");
    assert!(
        printed["mcpServers"]["aoe"].get("headers").is_none(),
        "a --no-auth daemon has no token to send: {printed}"
    );

    // The warning is accurate: with the setting off there is no `/mcp` route, and a `web`
    // build's GET-only dashboard fallback answers the POST with 405 instead of 404.
    let url = format!("http://127.0.0.1:{port}/mcp");
    let (status, _) = block_on(rpc(&url, &[], initialize()));
    assert!(
        matches!(
            status,
            StatusCode::NOT_FOUND | StatusCode::METHOD_NOT_ALLOWED
        ),
        "{status}"
    );
}
