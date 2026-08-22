use crate::process::RunFailure;
use std::fs;
use std::path::Path;

pub fn write_source(path: &Path, code: &str) -> Result<(), RunFailure> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| RunFailure::message(format!("failed to create source directory: {e}")))?;
    }
    fs::write(path, code)
        .map_err(|e| RunFailure::message(format!("failed to store stdin source: {e}")))
}

pub fn path_text(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}
pub fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|v| (*v).to_string()).collect()
}

pub fn uv_install_hint() -> String {
    if cfg!(target_os = "macos") {
        "Install uv with Homebrew: brew install uv".into()
    } else if cfg!(windows) {
        "Install uv with WinGet: winget install --id astral-sh.uv -e".into()
    } else {
        "Install uv: curl -LsSf https://astral.sh/uv/install.sh | sh".into()
    }
}
pub fn vp_install_hint() -> String {
    if cfg!(windows) {
        "Install vp in PowerShell: irm https://vite.plus/ps1 | iex".into()
    } else {
        "Install vp: curl -fsSL https://vite.plus | bash".into()
    }
}
pub fn rustup_install_hint() -> String {
    if cfg!(target_os = "macos") {
        "Install rustup with Homebrew: brew install rustup; then add $(brew --prefix rustup)/bin to PATH".into()
    } else if cfg!(windows) {
        "Install rustup from https://rustup.rs and select the stable toolchain".into()
    } else {
        "Install rustup: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh".into()
    }
}
pub fn mise_install_hint() -> String {
    if cfg!(target_os = "macos") {
        "Install mise with Homebrew: brew install mise".into()
    } else if cfg!(windows) {
        "Install mise with WinGet: winget install jdx.mise".into()
    } else {
        "Install mise: curl https://mise.run | sh".into()
    }
}

pub fn program_install_hint(program: &str) -> Option<String> {
    match program {
        "uv" => Some(uv_install_hint()),
        "vp" => Some(vp_install_hint()),
        "rustup" => Some(rustup_install_hint()),
        "mise" => Some(mise_install_hint()),
        _ => None,
    }
}
