use std::process::ExitCode;

use o2un_mcp_ping::PingServer;

fn main() -> ExitCode {
    o2un_mcp_core::run("ping", PingServer::new)
}
