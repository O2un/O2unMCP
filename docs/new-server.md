# 새 기능 서버 만들기

규칙과 구조는 [architecture.md](architecture.md). 기준 예제는 `servers/ping`.

## 1. 도구를 먼저 정한다

코드 전에 적는다:

- 도구 목록: 도구 하나에 동사 하나 (조회 / 제안 / 검증). 조회와 쓰기를 한 도구에 섞지 않는다.
- 각 도구의 입력, 출력, 실패 의미
- 호출 순서 → 서버 instructions가 된다
- 코드로 판정할 수 있는 검증(타입, 키 중복, 범위)은 도구가 한다. LLM에 맡기지 않는다.
- 원본 데이터는 바꾸지 않는다. 제안·스테이징 파일까지만 쓰고, 원본 반영은 사람이 승인한 쪽이 한다.

## 2. 뼈대 만들기

```
cargo xtask new-server excel-data
```

`servers/excel-data`가 생기고 workspace에 자동으로 들어간다 (`members = ["servers/*"]`).

| 이름 | 값 |
| --- | --- |
| 패키지 / exe | `o2un-mcp-excel-data` / `o2un-mcp-excel-data.exe` |
| 크레이트 | `o2un_mcp_excel_data` |
| 서버 타입 | `ExcelDataServer` |
| 환경변수 접두사 | `O2UN_EXCEL_DATA_` |

만든 직후 그대로 통과해야 한다. 통과하지 않으면 서버가 아니라 `templates/server`를 고친다.

```
cargo test -p o2un-mcp-excel-data
```

## 3. 채우기

1. `Cargo.toml`의 `description`
2. `src/config.rs`: 예시 `greeting`을 지우고 필요한 설정을 넣는다. 값 검증 예시는 `servers/ping/src/config.rs`. 설정이 없으면 `Config`를 비워 둔다.
3. `src/tools/`: `hello.rs`를 지우고 도구마다 파일 하나. 입력 구조체의 필드 `///` 주석은 입력 스키마 설명이 된다.
4. `src/server.rs`: 도구 메서드를 `#[tool(description = "...")]`로 추가하고, 본문은 `tools::x::run(..)` 호출만 둔다. `INSTRUCTIONS`에 호출 순서를 쓴다.
5. `tests/server.rs`: 도구마다 정상 1개, 오류(`is_error`) 1개 이상.

## 4. 확인

```
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

실제 클라이언트로 호출해 본다. 설정 파일의 키(`excel-data`)가 도구 이름 앞부분이 된다: `mcp__<키>__<도구>`.

```json
{
  "mcpServers": {
    "excel-data": {
      "type": "stdio",
      "command": "C:/path/to/target/release/o2un-mcp-excel-data.exe",
      "args": [],
      "env": { "O2UN_EXCEL_DATA_ROOT": "C:/data" }
    }
  }
}
```

```
claude -p "..." --mcp-config mcp.json --strict-mcp-config --allowedTools "mcp__excel-data__<도구>"
```

로그가 더 필요하면 `env`에 `"O2UN_MCP_LOG": "debug"`를 넣는다. 로그는 stderr로 나온다.
