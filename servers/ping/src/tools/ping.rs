//! `ping` 도구: 서버가 살아 있는지 확인한다.

use o2un_mcp_core::{ToolError, ToolResult};
use rmcp::schemars;
use serde::Deserialize;

use crate::config::Config;

/// `message` 최대 길이 (문자 수).
pub const MAX_MESSAGE_CHARS: usize = 1024;

/// `ping` 입력. 필드 문서 주석은 도구 입력 스키마의 description이 된다.
#[derive(Debug, Default, Deserialize, schemars::JsonSchema)]
pub struct PingParams {
    /// Optional text to echo back after the reply. At most 1024 characters.
    #[serde(default)]
    pub message: Option<String>,
}

/// `ping` 도구 본체.
///
/// 길이 제한은 LLM이 아니라 도구가 결정적으로 판정한다 (설계 원칙 3의 예시).
pub fn handle(config: &Config, params: PingParams) -> ToolResult<String> {
    match params.message {
        None => Ok(config.reply.clone()),
        Some(message) => {
            let len = message.chars().count();
            if len > MAX_MESSAGE_CHARS {
                return Err(ToolError::new(format!(
                    "message is {len} characters; the limit is {MAX_MESSAGE_CHARS}"
                )));
            }
            Ok(format!("{}: {message}", config.reply))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn echoes_message() {
        let params = PingParams {
            message: Some("hi".into()),
        };
        assert_eq!(handle(&Config::default(), params).unwrap(), "pong: hi");
    }

    #[test]
    fn limit_counts_characters_not_bytes() {
        let params = PingParams {
            message: Some("가".repeat(MAX_MESSAGE_CHARS)),
        };
        assert!(handle(&Config::default(), params).is_ok());
    }
}
