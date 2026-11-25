//! O2unMCP 서버들이 공유하는 뼈대.
//!
//! 코어가 맡는 것: 프로세스 시작·종료와 stdio 연결([`run`]), 설정 읽기 틀([`FromEnv`]),
//! 도구 에러를 MCP 결과로 바꾸는 규칙([`ToolError`]), 로그 초기화, 테스트 도우미.
//! 코어는 특정 기능(엑셀, UE 등)을 모른다. 도구와 그 의미는 각 서버가 정한다.

#![warn(missing_docs)]

mod config;
mod error;
mod log;
mod run;
#[cfg(feature = "testing")]
pub mod testing;

pub use config::{ConfigError, Env, FromEnv};
pub use error::{ToolError, ToolResult};
pub use log::{LOG_ENV, init_logging};
pub use run::{EXIT_CONFIG, EXIT_RUNTIME, RunError, run, serve_stdio};

/// 서버들이 `rmcp` 버전을 따로 고르지 않도록 코어가 쓰는 rmcp를 그대로 내보낸다.
pub use rmcp;

/// 도구를 켠 `ServerConfig`(초기화 응답)를 만든다.
///
/// 이름·버전은 **호출한 크레이트**의 `CARGO_PKG_NAME`/`CARGO_PKG_VERSION`이다.
/// `rmcp::model::Implementation::from_build_env()`는 rmcp 안에서 펼쳐져 이름이 "rmcp"가
/// 되므로 쓰지 않는다.
///
/// instructions는 필수다. 도구 호출 순서는 호출 쪽 하네스가 아니라 서버가 알려준다.
///
/// ```ignore
/// fn get_info(&self) -> rmcp::model::ServerConfig {
///     o2un_mcp_core::server_config!(INSTRUCTIONS)
/// }
/// ```
#[macro_export]
macro_rules! server_config {
    ($instructions:expr $(,)?) => {
        $crate::rmcp::model::ServerConfig::new(
            $crate::rmcp::model::ServerCapabilities::builder()
                .enable_tools()
                .build(),
        )
        .with_server_info($crate::rmcp::model::Implementation::new(
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
        ))
        .with_instructions($instructions)
    };
}
