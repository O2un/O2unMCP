//! MCP 표면: 도구 목록, instructions. 로직은 `tools/`에 있다.

use o2un_mcp_core::{ToolResult, server_config};
use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::ServerConfig,
    tool, tool_handler, tool_router,
};

use crate::config::Config;
use crate::tools::ping::{self, PingParams};

/// 클라이언트가 초기화 때 받는 작업 안내. 도구 호출 순서는 여기에 쓴다.
const INSTRUCTIONS: &str = "\
Health-check server. Call `ping` to confirm the server is reachable. \
Pass `message` to have it echoed back after the reply.";

/// ping MCP 서버. 상태는 불변 설정뿐이다 (서버는 턴마다 새로 뜬다).
#[derive(Clone)]
pub struct PingServer {
    config: Config,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl PingServer {
    /// 설정으로 서버를 만든다. `o2un_mcp_core::run`에 그대로 넘긴다.
    pub fn new(config: Config) -> Self {
        Self {
            config,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        description = "Reply with the configured text (default \"pong\"), followed by `message` if given."
    )]
    fn ping(&self, Parameters(params): Parameters<PingParams>) -> ToolResult<String> {
        ping::handle(&self.config, params)
    }
}

// 기본값은 호출마다 `Self::tool_router()`를 새로 만든다. 만들어 둔 라우터를 쓰도록 지정한다.
#[tool_handler(router = self.tool_router)]
impl ServerHandler for PingServer {
    fn get_info(&self) -> ServerConfig {
        server_config!(INSTRUCTIONS)
    }
}
