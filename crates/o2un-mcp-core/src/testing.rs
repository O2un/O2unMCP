//! 같은 프로세스 안에서 클라이언트와 서버를 잇는 테스트 도우미 (`testing` feature).
//!
//! 도구 함수를 직접 부르지 않고 실제 MCP `tools/call` 경로(직렬화, 스키마, isError 변환)를
//! 거쳐 시험하기 위해 쓴다. 테스트 전용이라 실패하면 바로 패닉한다.

use rmcp::{
    RoleClient, ServerHandler, ServiceExt,
    model::{CallToolRequestParams, CallToolResult},
    service::RunningService,
};
use serde_json::Value;
use tokio::task::JoinHandle;

/// 메모리 파이프 버퍼 크기. 테스트 메시지는 작아서 충분하다.
const PIPE_BUFFER: usize = 64 * 1024;

/// 서버에 연결된 테스트 클라이언트.
pub struct TestClient {
    client: RunningService<RoleClient, ()>,
    server: JoinHandle<()>,
}

/// 서버를 백그라운드 태스크로 띄우고 초기화 핸드셰이크까지 끝낸 클라이언트를 돌려준다.
pub async fn connect<S: ServerHandler>(server: S) -> TestClient {
    let (server_io, client_io) = tokio::io::duplex(PIPE_BUFFER);
    let server = tokio::spawn(async move {
        let running = server
            .serve(server_io)
            .await
            .expect("server failed to initialize");
        let _ = running.waiting().await;
    });
    let client = ().serve(client_io).await.expect("client failed to initialize");
    TestClient { client, server }
}

impl TestClient {
    /// 도구를 부른다. `args`는 JSON 객체(`json!({...})`)나 `Value::Null`(인자 없음).
    ///
    /// 도구가 `ToolError`를 돌려주면 패닉하지 않고 `is_error == Some(true)`인 결과가 온다.
    pub async fn call(&self, tool: &str, args: Value) -> CallToolResult {
        let mut params = CallToolRequestParams::new(tool.to_owned());
        match args {
            Value::Null => {}
            Value::Object(map) => params = params.with_arguments(map),
            other => panic!("tool arguments must be a JSON object or null, got {other}"),
        }
        self.client
            .call_tool(params)
            .await
            .unwrap_or_else(|e| panic!("tools/call {tool} failed at protocol level: {e}"))
    }

    /// 노출된 도구 이름 목록.
    pub async fn tool_names(&self) -> Vec<String> {
        self.client
            .list_all_tools()
            .await
            .expect("tools/list failed")
            .into_iter()
            .map(|tool| tool.name.into_owned())
            .collect()
    }

    /// 초기화 때 서버가 보낸 instructions.
    pub fn instructions(&self) -> Option<String> {
        self.client
            .peer_info()
            .and_then(|info| info.instructions.clone())
    }

    /// 초기화 때 서버가 밝힌 이름 (`server_config!`가 넣는 패키지 이름).
    pub fn server_name(&self) -> Option<String> {
        self.client
            .peer_info()
            .and_then(|info| info.server_info.as_ref().map(|i| i.name.clone()))
    }

    /// 연결을 닫고 서버 태스크가 끝날 때까지 기다린다.
    pub async fn close(self) {
        self.client.cancel().await.expect("client cancel failed");
        self.server.await.expect("server task panicked");
    }
}

/// 결과의 첫 텍스트 블록. 없으면 패닉.
pub fn text(result: &CallToolResult) -> &str {
    result
        .content
        .iter()
        .find_map(|block| block.as_text())
        .map(|t| t.text.as_str())
        .expect("tool result has no text content")
}

/// `is_error`가 켜졌는지.
pub fn is_error(result: &CallToolResult) -> bool {
    result.is_error == Some(true)
}
