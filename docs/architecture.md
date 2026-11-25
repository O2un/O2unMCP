# 아키텍처

코어(`crates/o2un-mcp-core`)와 기능 서버(`servers/<name>`)의 경계, 그리고 모든 서버가 따르는 규칙. 배경과 범위는 [blueprint.md](blueprint.md).

## 경계

| 코어가 맡는 것 | 서버가 맡는 것 |
| --- | --- |
| 프로세스 시작·종료, stdio 연결 (`run`, `serve_stdio`) | 도구 정의와 구현 |
| 설정 읽기와 검증 틀 (`FromEnv`, `Env`, `ConfigError`) | 서버별 설정 항목 |
| 에러를 MCP 결과로 바꾸는 규칙 (`ToolError`) | 도구별 에러 의미 |
| 로그 초기화 (`init_logging`) | 서버 instructions (작업 순서) |
| 초기화 응답 (`server_config!`) | |
| 테스트 도우미 (`testing`, feature) | 도구 테스트 |

코어는 특정 기능(엑셀, UE)을 모른다. 코어에 넣는 기준: 두 번째 서버도 그대로 쓰고, 기능 지식이 없을 것. 한 서버만 쓰는 도우미는 그 서버 안에 둔다.

## 서버 한 개의 모양

```
servers/<name>/
├─ Cargo.toml        package o2un-mcp-<name> → o2un-mcp-<name>.exe
├─ src/main.rs       o2un_mcp_core::run("<name>", XServer::new) 한 줄
├─ src/lib.rs        tests/에서 서버 타입을 쓰려고 lib으로 둔다
├─ src/config.rs     Config: FromEnv
├─ src/server.rs     #[tool_router] 도구 메서드(얇게) + #[tool_handler] get_info
├─ src/tools/*.rs    도구 로직. MCP와 무관한 순수 함수
└─ tests/*.rs        testing::connect로 실제 tools/call 경로 시험
```

새 서버는 직접 만들지 않고 `cargo xtask new-server <name>`으로 만든다 ([new-server.md](new-server.md)).

## 실행 흐름

`run(name, build)`:

1. `init_logging`: stderr, ANSI 없음, 필터는 `O2UN_MCP_LOG` (기본 `info`). 값이 틀리면 stderr에 알리고 기본값으로 돈다.
2. `C::from_env(&Env::process())`: 실패하면 stderr에 이유를 쓰고 **exit 2**. 도구는 하나도 노출되지 않는다.
3. tokio 런타임 안에서 `build(config)`로 서버를 만들고 `serve_stdio`를 실행한다.
4. 클라이언트가 stdin을 닫으면 정상 종료(exit 0). Ctrl+C는 취소 후 종료. 서비스 태스크가 실패하거나 초기화가 실패하면 **exit 1**.

## 규칙

### stdout
stdout은 MCP 프로토콜 전용이다. 서버·코어에서 `println!`/`print!`/`dbg!`를 쓰지 않는다. 로그는 tracing → stderr.

### 에러

| 상황 | 결과 |
| --- | --- |
| 도구가 `Err(ToolError::new(..))` | `isError: true` 도구 결과. 메시지가 그대로 LLM에 보인다 |
| 도구가 `Err(ToolError::internal("ctx", err))` | `isError: true`, 메시지는 `internal error: ctx`만. `err`는 stderr 로그 |
| 입력이 도구 스키마와 안 맞음 (타입 등) | rmcp가 `isError: true` 도구 결과로 돌려준다 (`failed to deserialize parameters: ...`). LLM이 읽고 고쳐 부를 수 있다 |
| 없는 도구 이름 | JSON-RPC `invalid_params` 프로토콜 에러 (`tool not found`) |
| 도구 안에서 패닉 | 프로세스는 살아 있고 그 요청에만 응답이 없다. 클라이언트는 타임아웃까지 기다린다. 도구 경로에 `unwrap`/`expect`를 두지 않는 이유 |

`ToolError::internal`의 context는 `&'static str`만 받는다. 경로나 사용자 값이 실수로 노출되지 않게 하기 위해서다.

### 설정
- 이름 규칙 `O2UN_<SERVER>_<KEY>`. MCP 클라이언트 설정의 `env`로 넘긴다.
- 빈 문자열은 "설정 안 됨"이다.
- 검증은 전부 시작 전에 한다. 도구 호출 중에 설정 오류가 드러나지 않게 한다.

### 상태
서버는 턴마다 새로 뜬다 (`claude -p --resume`도 마찬가지). 서버 구조체에는 불변 설정만 둔다. 턴 사이에 이어야 할 것은 파일(스테이징)로 넘긴다.

## rmcp

`=3.11.0`으로 고정한다 (태그 `rmcp-v3.11.0`). 3.x 안에서도 API가 바뀌므로 올릴 때는 그 태그의 `examples/`를 다시 읽는다.

알아둘 점:
- `Implementation::from_build_env()`는 rmcp 안에서 펼쳐져 서버 이름이 `rmcp`가 된다. `server_config!`를 쓴다.
- `#[tool_handler]`는 기본으로 호출마다 `Self::tool_router()`를 새로 만든다. 서버는 `#[tool_handler(router = self.tool_router)]`로 만들어 둔 라우터를 쓴다.
- 도구가 `Result<T, E>`를 돌려주고 `E: IntoContents`이면 `Err`는 `isError` 결과가 된다 (`ToolError`가 이것을 구현한다). `E = ErrorData`이면 프로토콜 에러가 된다.

## 테스트

`o2un-mcp-core`의 `testing` feature(dev-dependency에서만 켠다):

```rust
let client = connect(PingServer::new(Config::default())).await;
let result = client.call("ping", json!({ "message": "hi" })).await;
assert_eq!(testing::text(&result), "pong: hi");
client.close().await;
```

`tokio::io::duplex`로 같은 프로세스 안에서 서버와 클라이언트를 잇는다. 도구 함수만 직접 부르는 테스트로 대체하지 않는다 (직렬화·스키마·isError 변환을 거치지 않으므로).
