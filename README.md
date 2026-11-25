# O2unMCP

여러 기능의 MCP 서버를 같은 뼈대로 만들기 위한 Rust 워크스페이스. 공식 Rust SDK [`rmcp`](https://github.com/modelcontextprotocol/rust-sdk)를 쓴다. 서버는 런타임 설치 없이 단일 exe로 배포되는 stdio MCP 서버다.

| 위치 | 내용 |
| --- | --- |
| `crates/o2un-mcp-core` | 공용 뼈대: 실행, 설정, 에러, 로그, 테스트 도우미 |
| `servers/ping` | 예제이자 기준 서버 |
| `templates/server` | 새 서버 뼈대 |
| `xtask` | `cargo xtask new-server <name>` |

## 빠르게 써보기

```
cargo build --release
```

MCP 클라이언트 설정에 exe를 stdio 서버로 넣는다.

```json
{ "mcpServers": { "ping": { "type": "stdio", "command": "C:/.../target/release/o2un-mcp-ping.exe", "args": [] } } }
```

```
claude -p "ping 도구를 호출해줘" --mcp-config mcp.json --strict-mcp-config --allowedTools mcp__ping__ping
```

## 새 서버

```
cargo xtask new-server <name>
```

순서는 [docs/new-server.md](docs/new-server.md), 구조와 규칙은 [docs/architecture.md](docs/architecture.md), 배경은 [docs/blueprint.md](docs/blueprint.md).
