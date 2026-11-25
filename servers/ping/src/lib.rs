//! O2unMCP 예제이자 기준 서버. 새 서버의 모양은 `templates/server`가 이 구조를 따른다.

pub mod config;
mod server;
pub mod tools;

pub use config::Config;
pub use server::PingServer;
