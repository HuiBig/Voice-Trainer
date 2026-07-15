use std::path::{Path, PathBuf};

pub fn platform_key(os: &str, arch: &str) -> Option<&'static str> {
    match (os, arch) {
        ("windows", "x86_64") => Some("windows-x64"),
        ("macos", "aarch64") => Some("macos-arm64"),
        ("macos", "x86_64") => Some("macos-x64"),
        _ => None,
    }
}

pub fn current_platform_key() -> Option<&'static str> {
    platform_key(std::env::consts::OS, std::env::consts::ARCH)
}

pub fn python_relative_path_for(os: &str) -> &'static str {
    if os == "windows" {
        "python/python.exe"
    } else {
        "python/bin/python3"
    }
}

pub fn python_relative_path() -> &'static str {
    python_relative_path_for(std::env::consts::OS)
}

pub fn executable_name_for(os: &str, name: &str) -> String {
    if os == "windows" {
        format!("{name}.exe")
    } else {
        name.to_string()
    }
}

pub fn executable_name(name: &str) -> String {
    executable_name_for(std::env::consts::OS, name)
}

pub fn project_runtime_root(project_root: &Path) -> Option<PathBuf> {
    Some(
        project_root
            .join("runtime-local")
            .join(current_platform_key()?),
    )
}

pub fn bundled_runtime_root(resource_dir: &Path) -> PathBuf {
    resource_dir.join("runtime")
}

pub fn managed_runtime_root() -> Option<PathBuf> {
    if cfg!(target_os = "macos") {
        return std::env::var_os("HOME").map(|home| {
            PathBuf::from(home).join("Library/Application Support/VoiceTrainer/runtime/current")
        });
    }
    std::env::var_os("LOCALAPPDATA")
        .map(|directory| PathBuf::from(directory).join("VoiceTrainer/runtime/current"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_supported_desktop_platforms() {
        assert_eq!(platform_key("windows", "x86_64"), Some("windows-x64"));
        assert_eq!(platform_key("macos", "aarch64"), Some("macos-arm64"));
        assert_eq!(platform_key("macos", "x86_64"), Some("macos-x64"));
        assert_eq!(platform_key("linux", "x86_64"), None);
    }

    #[test]
    fn project_runtime_uses_current_platform_key() {
        let root = Path::new("project");
        if let Some(platform) = current_platform_key() {
            assert_eq!(
                project_runtime_root(root),
                Some(root.join("runtime-local").join(platform))
            );
        }
    }

    #[test]
    fn uses_native_executable_layouts() {
        assert_eq!(python_relative_path_for("windows"), "python/python.exe");
        assert_eq!(python_relative_path_for("macos"), "python/bin/python3");
        assert_eq!(executable_name_for("windows", "ffmpeg"), "ffmpeg.exe");
        assert_eq!(executable_name_for("macos", "ffmpeg"), "ffmpeg");
    }
}
