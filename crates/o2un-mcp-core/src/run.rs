//! 프로세스 시작·종료와 stdio 연결.

use std::process::ExitCode;

use rmcp::{
    ServerHandler, ServiceExt,
    service::{QuitReason, ServerInitializeError},
    transport::stdio,
};

use crate::config::{Env, FromEnv};
use crate::log::init_logging;

/// 설정이 틀려 시작하지 않았을 때의 종료 코드.
pub const EXIT_CONFIG: u8 = 2;
/// 실행 중 실패(초기화 핸드셰이크 실패, 런타임 생성 실패 등)의 종료 코드.
pub const EXIT_RUNTIME: u8 = 1;

/// 서버 실행 실패.
#[derive(Debug, thiserror::Error)]
pub enum RunError {
    /// 초기화 핸드셰이크 실패 (클라이언트가 바로 끊었거나 프로토콜 불일치).
    #[error("failed to start MCP service: {0}")]
    // 초기화 에러는 커서(수백 바이트) 박스에 담는다. 원인 체인은 그대로 남는다.
    Start(#[source] Box<ServerInitializeError>),
    /// 서비스 태스크가 패닉 등으로 비정상 종료.
    #[error("MCP service task failed: {0}")]
    Join(#[from] tokio::task::JoinError),
}

/// 서버 `main`의 전부. 로그 → 설정 → 런타임 → stdio 서비스 순서로 진행한다.
///
/// ```ignore
/// fn main() -> std::process::ExitCode {
///     o2un_mcp_core::run("ping", PingServer::new)
/// }
/// ```
///
/// 설정이 틀리면 이유를 stderr에 쓰고 [`EXIT_CONFIG`]로 끝낸다.
pub fn run<C, S>(name: &str, build: impl FnOnce(C) -> S) -> ExitCode
where
    C: FromEnv,
    S: ServerHandler,
{
    init_logging(name);

    let config = match C::from_env(&Env::process()) {
        Ok(config) => config,
        Err(err) => {
            // tracing 필터가 꺼져 있어도 이유는 반드시 보이게 eprintln을 쓴다.
            eprintln!("{name}: configuration error: {err}");
            return ExitCode::from(EXIT_CONFIG);
        }
    };

    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(err) => {
            eprintln!("{name}: failed to create tokio runtime: {err}");
            return ExitCode::from(EXIT_RUNTIME);
        }
    };

    // 서버 생성자가 tokio를 써도 되도록 런타임 안에서 만든다.
    match runtime.block_on(async { serve_stdio(build(config)).await }) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{name}: {err}");
            ExitCode::from(EXIT_RUNTIME)
        }
    }
}

/// stdin/stdout으로 서버를 돌린다. 클라이언트가 stdin을 닫으면(턴 종료) 정상 종료,
/// Ctrl+C면 서비스를 취소하고 종료한다.
pub async fn serve_stdio<S: ServerHandler>(server: S) -> Result<(), RunError> {
    let service = server
        .serve(stdio())
        .await
        .map_err(|e| RunError::Start(Box::new(e)))?;
    tracing::info!("serving on stdio");

    let cancel = service.cancellation_token();
    tokio::select! {
        result = service.waiting() => match result? {
            QuitReason::JoinError(err) => return Err(RunError::Join(err)),
            reason => tracing::info!(?reason, "service stopped"),
        },
        () = ctrl_c() => {
            tracing::info!("ctrl-c received, shutting down");
            cancel.cancel();
        }
    }
    Ok(())
}

/// Ctrl+C를 기다린다. 핸들러를 못 걸면 끝나지 않는다: 그 경우 서버가 곧바로
/// 종료되지 않고 stdin이 닫힐 때까지 계속 돈다.
async fn ctrl_c() {
    if let Err(err) = tokio::signal::ctrl_c().await {
        tracing::warn!(error = %err, "cannot listen for ctrl-c; relying on stdin close");
        std::future::pending::<()>().await;
    }
}
