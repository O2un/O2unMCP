//! 워크스페이스 작업 도구. `cargo xtask <명령>`으로 실행한다 (`.cargo/config.toml` alias).
//!
//! 명령:
//!   new-server <name>   templates/server를 복사해 servers/<name>을 만든다.
//!
//! 루트 `Cargo.toml`의 `members = ["servers/*"]` 덕분에 디렉터리를 만드는 것만으로
//! workspace에 등록된다.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const USAGE: &str = "usage: cargo xtask new-server <name>

  <name>  lowercase letters, digits and single hyphens, starting with a letter
          (e.g. excel-data). Creates servers/<name>, package o2un-mcp-<name>.";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["new-server", name] => new_server(name),
        ["-h" | "--help" | "help"] | [] => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        _ => Err(format!(
            "unrecognized arguments: {}\n\n{USAGE}",
            args.join(" ")
        )),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("xtask: {err}");
            ExitCode::FAILURE
        }
    }
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives one level below the workspace root")
        .to_path_buf()
}

fn new_server(name: &str) -> Result<(), String> {
    validate_name(name)?;
    let root = workspace_root();
    let template = root.join("templates").join("server");
    let dest = root.join("servers").join(name);
    if dest.exists() {
        return Err(format!("{} already exists", dest.display()));
    }
    let names = Names::new(name);
    // 패키지 이름이 crates/의 라이브러리와 겹치면 workspace 전체가 깨진다 (예: core).
    if root.join("crates").join(&names.package).exists() {
        return Err(format!(
            "package {} already exists under crates/; choose another name",
            names.package
        ));
    }

    if let Err(err) = copy_rendered(&template, &dest, &names) {
        // 반쯤 만들어진 서버가 workspace 빌드를 깨지 않도록 지운다.
        let _ = fs::remove_dir_all(&dest);
        return Err(err);
    }

    // 이름 길이에 따라 줄바꿈·import 순서가 달라지므로 생성물을 포맷해 둔다.
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let formatted = Command::new(cargo)
        .args(["fmt", "--package", &names.package])
        .current_dir(&root)
        .status()
        .is_ok_and(|s| s.success());
    if !formatted {
        // fmt가 실패하면 대개 생성물이 workspace를 깨뜨린 것이다. 남겨두면 모든 cargo 명령이 실패한다.
        let _ = fs::remove_dir_all(&dest);
        return Err(format!(
            "cargo fmt failed on the generated package; removed servers/{name}"
        ));
    }

    println!("created servers/{name} (package {})", names.package);
    println!("next: cargo test -p {}", names.package);
    println!("then: docs/new-server.md");
    Ok(())
}

/// 템플릿 플레이스홀더에 들어갈 이름들.
struct Names {
    name: String,
    package: String,
    krate: String,
    type_name: String,
    env: String,
}

impl Names {
    fn new(name: &str) -> Self {
        let package = format!("o2un-mcp-{name}");
        Self {
            name: name.to_owned(),
            krate: package.replace('-', "_"),
            package,
            type_name: name.split('-').map(capitalize).collect(),
            env: name.replace('-', "_").to_ascii_uppercase(),
        }
    }

    fn render(&self, text: &str) -> String {
        text.replace("{{name}}", &self.name)
            .replace("{{package}}", &self.package)
            .replace("{{crate}}", &self.krate)
            .replace("{{Type}}", &self.type_name)
            .replace("{{ENV}}", &self.env)
    }
}

fn capitalize(part: &str) -> String {
    let mut chars = part.chars();
    match chars.next() {
        Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
        None => String::new(),
    }
}

fn validate_name(name: &str) -> Result<(), String> {
    let valid = name.starts_with(|c: char| c.is_ascii_lowercase())
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !name.ends_with('-')
        && !name.contains("--");
    if valid {
        Ok(())
    } else {
        Err(format!("invalid server name {name:?}\n\n{USAGE}"))
    }
}

/// `src`를 재귀 복사하면서 내용의 플레이스홀더를 바꾸고 `.tmpl` 확장자를 뗀다.
/// 템플릿의 `Cargo.toml`은 `Cargo.toml.tmpl`로 둬서 cargo가 패키지로 잡지 않게 한다.
fn copy_rendered(src: &Path, dest: &Path, names: &Names) -> Result<(), String> {
    fs::create_dir_all(dest).map_err(|e| format!("create {}: {e}", dest.display()))?;
    let entries = fs::read_dir(src).map_err(|e| format!("read {}: {e}", src.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("read {}: {e}", src.display()))?;
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy().into_owned();
        let target = dest.join(file_name.strip_suffix(".tmpl").unwrap_or(&file_name));
        if path.is_dir() {
            copy_rendered(&path, &target, names)?;
        } else {
            let text =
                fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
            fs::write(&target, names.render(&text))
                .map_err(|e| format!("write {}: {e}", target.display()))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_names_from_hyphenated_name() {
        let names = Names::new("excel-data");
        assert_eq!(names.package, "o2un-mcp-excel-data");
        assert_eq!(names.krate, "o2un_mcp_excel_data");
        assert_eq!(names.type_name, "ExcelData");
        assert_eq!(names.env, "EXCEL_DATA");
    }

    #[test]
    fn validates_names() {
        for ok in ["ping", "excel-data", "v2", "a1-b2"] {
            assert!(validate_name(ok).is_ok(), "{ok}");
        }
        for bad in ["", "Ping", "1ping", "-ping", "ping-", "a--b", "a_b", "a b"] {
            assert!(validate_name(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn template_leaves_no_placeholders() {
        let names = Names::new("demo");
        let template = workspace_root().join("templates").join("server");
        let mut stack = vec![template];
        while let Some(dir) = stack.pop() {
            for entry in fs::read_dir(&dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    stack.push(path);
                } else {
                    let rendered = names.render(&fs::read_to_string(&path).unwrap());
                    assert!(
                        !rendered.contains("{{"),
                        "unknown placeholder left in {}",
                        path.display()
                    );
                }
            }
        }
    }
}
