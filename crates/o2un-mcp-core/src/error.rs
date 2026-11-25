//! 도구 실패를 MCP `isError` 결과로 바꾸는 규칙.
//!
//! 도구가 `ToolResult<T>`를 반환하면 rmcp가 `Err(ToolError)`를 `is_error: true`인
//! `CallToolResult`로 바꾼다. 프로토콜 에러가 아니라 도구 결과라서, LLM이 메시지를 읽고
//! 입력을 고쳐 다시 부를 수 있다.

use std::fmt;

use rmcp::model::{ContentBlock, IntoContents};

/// 도구 호출 실패. 담긴 메시지는 그대로 클라이언트(LLM)에게 보인다.
///
/// 내부 에러(io, 경로, 하위 라이브러리 메시지)를 그대로 넣지 않는다.
/// 그런 에러는 [`ToolError::internal`]로 감싸면 로그로만 남고 노출되지 않는다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolError {
    message: String,
}

impl ToolError {
    /// 도구가 의미를 정한 실패. 예: `"row key 'A01' already exists"`.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// 예상하지 못한 내부 실패. `err`는 stderr 로그에만 남기고,
    /// 클라이언트에는 `context`만 보인다.
    ///
    /// `context`가 `&'static str`인 것은 의도다. 경로·사용자 값 같은 동적 문자열이
    /// 실수로 노출되지 않게 고정 문구만 받는다.
    pub fn internal(context: &'static str, err: impl fmt::Display) -> Self {
        tracing::error!(context, error = %err, "internal tool error");
        Self {
            message: format!("internal error: {context}"),
        }
    }

    /// 클라이언트에 보일 메시지.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ToolError {}

impl IntoContents for ToolError {
    fn into_contents(self) -> Vec<ContentBlock> {
        vec![ContentBlock::text(self.message)]
    }
}

/// 도구 함수의 반환 타입.
pub type ToolResult<T> = Result<T, ToolError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn internal_hides_source_error() {
        let err = ToolError::internal("reading schema", "C:/secret/path.xlsx: access denied");
        assert_eq!(err.message(), "internal error: reading schema");
    }
}
