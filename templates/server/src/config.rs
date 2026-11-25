//! {{name}} 서버 설정. MCP 클라이언트 설정의 `env`로 받는다.
//! 이름 규칙: `O2UN_{{ENV}}_<KEY>`. 틀리면 서버는 시작하지 않는다.

use o2un_mcp_core::{ConfigError, Env, FromEnv};

/// 예시 설정. 선택, 기본값 [`DEFAULT_GREETING`]. 필요 없으면 지우고 `Config`를 비운다.
pub const GREETING_ENV: &str = "O2UN_{{ENV}}_GREETING";
/// [`GREETING_ENV`]가 없을 때의 인사말.
pub const DEFAULT_GREETING: &str = "Hello";

/// {{name}} 서버 설정. 시작 전에 [`FromEnv`]로 한 번 읽고 바뀌지 않는다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// 예시 값.
    pub greeting: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            greeting: DEFAULT_GREETING.to_owned(),
        }
    }
}

impl FromEnv for Config {
    fn from_env(env: &Env) -> Result<Self, ConfigError> {
        Ok(Self {
            greeting: env
                .optional(GREETING_ENV)
                .unwrap_or_else(|| DEFAULT_GREETING.to_owned()),
        })
    }
}
