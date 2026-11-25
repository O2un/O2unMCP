use std::process::ExitCode;

use {{crate}}::{{Type}}Server;

fn main() -> ExitCode {
    o2un_mcp_core::run("{{name}}", {{Type}}Server::new)
}
