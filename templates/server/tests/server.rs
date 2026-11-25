//! 실제 MCP 경로(tools/list, tools/call)로 서버를 시험한다. 도구마다 정상·오류 테스트를 둔다.

use o2un_mcp_core::testing::{self, connect};
use {{crate}}::{Config, {{Type}}Server};
use serde_json::json;

fn server() -> {{Type}}Server {
    {{Type}}Server::new(Config::default())
}

#[tokio::test]
async fn exposes_tools_with_identity_and_instructions() {
    let client = connect(server()).await;
    assert_eq!(client.tool_names().await, ["hello"]);
    assert_eq!(client.server_name().as_deref(), Some("{{package}}"));
    assert!(client.instructions().is_some_and(|s| s.contains("hello")));
    client.close().await;
}

#[tokio::test]
async fn hello_greets() {
    let client = connect(server()).await;
    let result = client.call("hello", json!({ "name": "O2un" })).await;
    assert!(!testing::is_error(&result));
    assert_eq!(testing::text(&result), "Hello, O2un!");
    client.close().await;
}

#[tokio::test]
async fn hello_rejects_empty_name() {
    let client = connect(server()).await;
    let result = client.call("hello", json!({ "name": "  " })).await;
    assert!(testing::is_error(&result));
    client.close().await;
}
