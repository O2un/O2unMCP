//! 환경변수 → 타입 있는 설정.
//!
//! 서버는 MCP 클라이언트 설정의 `env`로 값을 받는다. 시작 전에 전부 읽고 검증해서,
//! 틀리면 도구를 하나도 노출하지 않고 종료한다 (설정 오류가 도구 호출 중에 드러나지 않게).
//! 이름 규칙: `O2UN_<SERVER>_<KEY>`.

use std::collections::HashMap;
use std::fmt::Display;
use std::str::FromStr;

/// 설정 로드 실패. 메시지는 stderr에 그대로 출력된다.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ConfigError {
    /// 필수 변수가 없거나 비어 있음.
    #[error("required environment variable {key} is not set")]
    Missing {
        /// 환경변수 이름.
        key: String,
    },
    /// 값은 있지만 쓸 수 없음 (파싱 실패, 범위 밖 등).
    #[error("environment variable {key} is invalid: {reason}")]
    Invalid {
        /// 환경변수 이름.
        key: String,
        /// 사람이 읽을 이유. stderr에 그대로 나간다.
        reason: String,
    },
}

/// 서버별 설정 타입이 구현한다.
pub trait FromEnv: Sized {
    /// 설정 전체를 읽고 검증한다. 하나라도 틀리면 서버는 시작하지 않는다.
    fn from_env(env: &Env) -> Result<Self, ConfigError>;
}

/// 설정이 필요 없는 서버용.
impl FromEnv for () {
    fn from_env(_: &Env) -> Result<Self, ConfigError> {
        Ok(())
    }
}

/// 환경변수 읽기. 테스트에서는 [`Env::from_pairs`]로 프로세스 환경 대신 맵을 쓴다.
///
/// 빈 문자열은 "설정 안 됨"으로 본다. MCP 설정 파일에서 `""`로 남겨둔 값을 기본값 대신
/// 쓰는 실수를 막기 위해서다.
#[derive(Debug, Clone, Default)]
pub struct Env {
    vars: HashMap<String, String>,
}

impl Env {
    /// 현재 프로세스의 환경변수. UTF-8이 아닌 값은 건너뛴다.
    pub fn process() -> Self {
        Self {
            vars: std::env::vars_os()
                .filter_map(|(k, v)| Some((k.into_string().ok()?, v.into_string().ok()?)))
                .collect(),
        }
    }

    /// 주어진 쌍만 담은 환경. 테스트에서 프로세스 환경 대신 쓴다.
    pub fn from_pairs<I, K, V>(pairs: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        Self {
            vars: pairs
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        }
    }

    /// 값이 있으면 `Some`. 빈 문자열과 UTF-8이 아닌 값은 `None`.
    pub fn optional(&self, key: &str) -> Option<String> {
        self.vars.get(key).filter(|v| !v.is_empty()).cloned()
    }

    /// 값이 없으면 [`ConfigError::Missing`].
    pub fn required(&self, key: &str) -> Result<String, ConfigError> {
        self.optional(key).ok_or_else(|| ConfigError::Missing {
            key: key.to_owned(),
        })
    }

    /// 값이 있으면 `T`로 파싱한다. 파싱 실패는 [`ConfigError::Invalid`].
    pub fn optional_parse<T>(&self, key: &str) -> Result<Option<T>, ConfigError>
    where
        T: FromStr,
        T::Err: Display,
    {
        self.optional(key).map(|raw| parse(key, &raw)).transpose()
    }

    /// 값을 `T`로 파싱한다. 없으면 `Missing`, 파싱 실패면 `Invalid`.
    pub fn required_parse<T>(&self, key: &str) -> Result<T, ConfigError>
    where
        T: FromStr,
        T::Err: Display,
    {
        parse(key, &self.required(key)?)
    }
}

fn parse<T>(key: &str, raw: &str) -> Result<T, ConfigError>
where
    T: FromStr,
    T::Err: Display,
{
    raw.parse().map_err(|e: T::Err| ConfigError::Invalid {
        key: key.to_owned(),
        reason: format!("{e} (value: {raw:?})"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_value_counts_as_missing() {
        let env = Env::from_pairs([("A", "")]);
        assert_eq!(env.optional("A"), None);
        assert_eq!(
            env.required("A"),
            Err(ConfigError::Missing { key: "A".into() })
        );
    }

    #[test]
    fn parse_reports_key_and_value() {
        let env = Env::from_pairs([("PORT", "abc")]);
        let err = env.required_parse::<u16>("PORT").unwrap_err();
        let ConfigError::Invalid { key, reason } = err else {
            panic!("expected Invalid, got {err:?}");
        };
        assert_eq!(key, "PORT");
        assert!(reason.contains("\"abc\""), "{reason}");
    }

    #[test]
    fn optional_parse_absent_is_none() {
        let env = Env::default();
        assert_eq!(env.optional_parse::<u16>("PORT"), Ok(None));
    }
}
