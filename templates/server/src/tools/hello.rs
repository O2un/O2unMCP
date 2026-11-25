//! `hello` 예시 도구. 첫 실제 도구를 만들면 지운다.

use o2un_mcp_core::{ToolError, ToolResult};
use rmcp::schemars;
use serde::Deserialize;

use crate::config::Config;

/// 필드 문서 주석은 도구 입력 스키마의 description이 된다.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct HelloParams {
    /// Who to greet. Must not be empty.
    pub name: String,
}

/// `hello` 도구 본체. 빈 이름은 도구가 결정적으로 거절한다.
pub fn run(config: &Config, params: HelloParams) -> ToolResult<String> {
    let name = params.name.trim();
    if name.is_empty() {
        return Err(ToolError::new("name must not be empty"));
    }
    Ok(format!("{}, {name}!", config.greeting))
}
