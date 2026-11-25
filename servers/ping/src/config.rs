//! ping 서버 설정. MCP 클라이언트 설정의 `env`로 받는다.

use o2un_mcp_core::{ConfigError, Env, FromEnv};

/// 응답 앞에 붙는 문자열. 선택, 기본값 [`DEFAULT_REPLY`].
pub const REPLY_ENV: &str = "O2UN_PING_REPLY";
/// [`REPLY_ENV`]가 없을 때의 응답.
pub const DEFAULT_REPLY: &str = "pong";
const MAX_REPLY_CHARS: usize = 64;

/// ping 서버 설정. 시작 전에 [`FromEnv`]로 한 번 읽고 바뀌지 않는다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// 응답 앞에 붙는 문자열 (최대 64자).
    pub reply: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            reply: DEFAULT_REPLY.to_owned(),
        }
    }
}

impl FromEnv for Config {
    fn from_env(env: &Env) -> Result<Self, ConfigError> {
        let Some(reply) = env.optional(REPLY_ENV) else {
            return Ok(Self::default());
        };
        if reply.chars().count() > MAX_REPLY_CHARS {
            return Err(ConfigError::Invalid {
                key: REPLY_ENV.to_owned(),
                reason: format!("must be at most {MAX_REPLY_CHARS} characters"),
            });
        }
        Ok(Self { reply })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_pong() {
        let config = Config::from_env(&Env::default()).unwrap();
        assert_eq!(config.reply, DEFAULT_REPLY);
    }

    #[test]
    fn rejects_long_reply() {
        let env = Env::from_pairs([(REPLY_ENV, "x".repeat(MAX_REPLY_CHARS + 1))]);
        assert!(matches!(
            Config::from_env(&env),
            Err(ConfigError::Invalid { .. })
        ));
    }
}
