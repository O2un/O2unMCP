//! 실제 MCP 경로(tools/list, tools/call)로 ping 서버를 시험한다.

use o2un_mcp_core::testing::{self, connect};
use o2un_mcp_ping::{Config, PingServer, tools::ping::MAX_MESSAGE_CHARS};
use serde_json::{Value, json};

fn server() -> PingServer {
    PingServer::new(Config::default())
}

#[tokio::test]
async fn exposes_ping_with_identity_and_instructions() {
    let client = connect(server()).await;
    assert_eq!(client.tool_names().await, ["ping"]);
    assert_eq!(client.server_name().as_deref(), Some("o2un-mcp-ping"));
    assert!(client.instructions().is_some_and(|s| s.contains("ping")));
    client.close().await;
}

#[tokio::test]
async fn replies_pong_without_arguments() {
    let client = connect(server()).await;
    let result = client.call("ping", Value::Null).await;
    assert!(!testing::is_error(&result));
    assert_eq!(testing::text(&result), "pong");
    client.close().await;
}

#[tokio::test]
async fn echoes_message_with_configured_reply() {
    let client = connect(PingServer::new(Config {
        reply: "ack".into(),
    }))
    .await;
    let result = client.call("ping", json!({ "message": "hi" })).await;
    assert_eq!(testing::text(&result), "ack: hi");
    client.close().await;
}

#[tokio::test]
async fn too_long_message_is_tool_error() {
    let client = connect(server()).await;
    let message = "x".repeat(MAX_MESSAGE_CHARS + 1);
    let result = client.call("ping", json!({ "message": message })).await;
    assert!(testing::is_error(&result));
    let expected = format!("limit is {MAX_MESSAGE_CHARS}");
    assert!(testing::text(&result).contains(&expected));
    client.close().await;
}
