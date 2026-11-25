# O2unMCP 청사진

작성 2026-11-25. 아직 코드는 없다. 이 문서를 기준으로 구현을 시작한다.

## 왜 만드나

[DataDriven4UE5](https://github.com/O2un/DataDriven4UE5)(엑셀 데이터 드리븐 UE 플러그인)에 AI로 데이터를 채우고 데이터 구조를 설계하는 기능을 붙이려고 한다. 에디터가 구독형 AI CLI(`claude -p`)를 띄우고, CLI는 MCP 서버의 도구를 호출해 엑셀 수준의 일을 한다.

MCP 서버를 어디에 둘지는 스파이크로 정했다(2026-11-25, DataDriven4UE5 쪽 기록).

| 방식 | 결과 |
| --- | --- |
| A. UE 5.8 공식 MCP에 C++ 툴셋 등록 | 동작. 기본 설정에서 턴 2배, 엔진 툴셋 18개가 같이 노출, Experimental·NoRedist 플러그인 의존 |
| **B. 외부 stdio MCP 서버** | 동작. 우리 도구만 노출, 에디터 없이 테스트 가능, codex 등 다른 CLI에도 연결 가능 → **채택** |

여러 턴 대화(`claude -p --resume <session_id>`)에서도 B가 동작했다. stdio 서버는 턴마다 새로 뜬다.

이 서버를 한 기능 전용으로 만들지 않고, 앞으로 여러 기능 서버를 같은 뼈대로 만들 수 있게 레포를 따로 둔다.

## 목표

- 공용 코어 라이브러리 하나와, 기능마다 독립된 서버 실행 파일
- 새 기능 서버를 명령 하나로 뼈대부터 만들 수 있다
- 서버는 런타임 설치 없이 단일 exe로 배포된다

## 기술 선택

| 항목 | 선택 | 근거 |
| --- | --- | --- |
| 언어 | Rust (edition 2024) | 단일 exe 배포, 네이티브 DLL FFI, 작성자가 학습 중 |
| MCP SDK | [`rmcp`](https://github.com/modelcontextprotocol/rust-sdk) 3.11.0 | 공식 Rust SDK. 2026-11-25 확인: 레포 활성, 3.11.0이 2026-11-17 릴리스 |
| 비동기 | tokio | rmcp 기본 |
| 로그 | tracing → stderr | stdout은 MCP 프로토콜 전용 |
| 테스트 | cargo test, 같은 프로세스 안 클라이언트-서버 연결 | |
| 툴체인 | stable-x86_64-pc-windows-msvc (작성 시 rustc 1.99.1) | UE·DLL과 같은 MSVC ABI |

> rmcp는 3.x 안에서도 API가 바뀐다. 구현할 때 고정한 버전 태그의 예제를 직접 읽고 맞춘다.

선례: devlog의 [writer](https://github.com/O2un/writer)는 TypeScript MCP 서버다. 당시에는 Rust 공식 SDK가 없어 TypeScript를 썼다. 설계 원칙은 writer에서 가져온다. writer 자체를 옮기지는 않는다.

## 구조

```
O2unMCP/                          cargo workspace
├─ crates/o2un-mcp-core/          lib — 서버들이 공유하는 뼈대
│   ├─ run.rs                     run_stdio(server): tokio 런타임, stdio 전송, 종료 처리
│   ├─ config.rs                  환경변수 → 타입 있는 설정. 잘못되면 ConfigError로 시작 전에 종료하고 stderr에 이유 출력
│   ├─ error.rs                   ToolError → MCP isError 결과. 내부 에러를 그대로 노출하지 않음
│   ├─ log.rs                     tracing 초기화 (stderr)
│   └─ testing.rs                 같은 프로세스 안에서 클라이언트와 서버를 잇는 테스트 도우미
├─ servers/
│   └─ ping/                      bin — 예제이자 기준 서버. ping 도구 1개 + 테스트
├─ templates/server/              새 서버 뼈대 (Cargo.toml, main.rs, server.rs, tools/, tests/)
├─ xtask/                         cargo xtask new-server <name> → templates/server를 복사해 servers/<name> 생성, workspace에 등록
├─ docs/
│   ├─ blueprint.md               이 문서
│   ├─ architecture.md            코어와 서버의 경계, 설계 원칙 (구현과 함께 작성)
│   └─ new-server.md              새 기능 서버를 만드는 순서
└─ .github/workflows/ci.yml       fmt --check · clippy -D warnings · test · build --release (windows-latest)
```

### 코어와 서버의 경계

| 코어가 맡는 것 | 서버가 맡는 것 |
| --- | --- |
| 프로세스 시작·종료, stdio 연결 | 도구 정의와 구현 |
| 설정 읽기와 검증 틀 | 서버별 설정 항목 |
| 에러를 MCP 결과로 바꾸는 규칙 | 도구별 에러 의미 |
| 로그 초기화 | 서버 instructions (작업 순서) |
| 테스트 도우미 | 도구 테스트 |

코어는 특정 기능(엑셀, UE)을 모른다.

## 설계 원칙 (writer에서 가져옴)

1. **서버는 상태를 갖지 않는다.** CLI가 턴마다 서버를 새로 띄운다. 턴 사이에 이어야 할 것은 파일(스테이징)로 넘긴다.
2. **사람이 보지 않은 변경은 원본에 들어가지 않는다.** 도구는 제안과 스테이징까지만 쓴다. 원본 반영은 승인한 쪽이 한다.
3. **코드로 판정할 수 있는 것은 LLM에 맡기지 않는다.** 타입·키 중복·범위 같은 검증은 도구가 결정적으로 한다.
4. **작업 순서는 서버 instructions에 적는다.** 호출한 쪽의 하네스(스킬, CLAUDE.md)에 기대지 않는다.
5. **설정이 틀리면 시작하지 않는다.** 이유를 stderr에 적고 종료한다.

## 범위

### 1단계 (이 청사진의 구현 범위)

- `o2un-mcp-core`
- `servers/ping` (예제이자 기준 서버)
- `templates/server` + `cargo xtask new-server`
- 문서, CI

### 이후

| 단계 | 내용 |
| --- | --- |
| 2 | `servers/excel-data` (가칭) — DataDriven4UE5용. ExcelHelper DLL FFI, 스키마 조회·행 읽기·행 제안·스키마 제안, 스테이징 |
| 미정 | LLM Provider(`claude-code`, `anthropic-api`) — 서버 안에서 LLM을 부를 기능이 생기면 |

## 검증 (1단계 완료 기준)

- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo build --release` 통과
- `cargo xtask new-server demo`로 만든 서버가 수정 없이 빌드·테스트 통과
- 실제 클라이언트 호출: `claude -p --mcp-config <cfg> --strict-mcp-config --allowedTools mcp__ping__ping`로 `ping.exe`의 결과를 받음

## 사용하는 쪽과의 연결

AI CLI의 MCP 설정에 서버 exe를 stdio 서버로 넣는다.

```json
{ "mcpServers": { "o2un-ping": { "type": "stdio", "command": "C:/.../ping.exe", "args": [] } } }
```

DataDriven4UE5에서는 UE가 `claude -p ... --mcp-config <이 파일>`을 비동기로 띄운다. exe 배포 위치(플러그인 `Binaries/ThirdParty` 등)는 2단계에서 정한다.

## 정해야 할 것

- [ ] 라이선스 (DataDriven4UE5는 AGPL-3.0-or-later)
- [ ] 이 레포의 하네스: 이 레포에서 따로 세팅
- [ ] 서버 exe 이름 규칙 (`o2un-mcp-<name>.exe` 등)
- [ ] 릴리스 방식 (GitHub Release에 exe 첨부 여부)
