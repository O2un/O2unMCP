//! 로그 초기화. stdout은 MCP 프로토콜 전용이라 로그는 반드시 stderr로 보낸다.

use tracing_subscriber::EnvFilter;

/// 로그 필터를 읽는 환경변수. 예: `O2UN_MCP_LOG=debug`, `O2UN_MCP_LOG=o2un_mcp_ping=trace`.
pub const LOG_ENV: &str = "O2UN_MCP_LOG";
const DEFAULT_FILTER: &str = "info";

/// stderr 로그를 켠다. 두 번 불러도 패닉하지 않는다 (두 번째 호출은 무시).
pub fn init_logging(name: &str) {
    // 변수가 없으면 기본값. 값이 틀리면 조용히 넘어가지 않고 이유를 알린 뒤 기본값으로 돈다
    // (로그 설정 오류로 서버를 멈추지는 않는다).
    // 빈 값은 "설정 안 됨"이다 (Env와 같은 규칙). EnvFilter는 ""를 "전부 끔"으로 받아들여
    // ToolError::internal의 원인 로그까지 사라지므로 따로 거른다.
    let filter = match std::env::var(LOG_ENV) {
        Ok(raw) if raw.trim().is_empty() => EnvFilter::new(DEFAULT_FILTER),
        Err(_) => EnvFilter::new(DEFAULT_FILTER),
        Ok(raw) => EnvFilter::try_new(&raw).unwrap_or_else(|err| {
            eprintln!("{name}: invalid {LOG_ENV}={raw:?} ({err}); using {DEFAULT_FILTER:?}");
            EnvFilter::new(DEFAULT_FILTER)
        }),
    };
    let installed = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        // MCP 클라이언트는 stderr를 파일·패널에 그대로 남기므로 색 코드를 넣지 않는다.
        .with_ansi(false)
        .try_init()
        .is_ok();
    if installed {
        tracing::debug!(server = name, "logging initialized");
    }
}
