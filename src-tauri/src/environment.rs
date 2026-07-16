use crate::runtime_paths;
use serde::Serialize;
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
use sysinfo::{Disks, System};
use tauri::Manager;

const GIB: u64 = 1024 * 1024 * 1024;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentReport {
    checked_at: u64,
    platform: PlatformInfo,
    resources: ResourceInfo,
    dependencies: Vec<DependencyCheck>,
    summary: EnvironmentSummary,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PlatformInfo {
    os: String,
    os_version: String,
    architecture: String,
    host_name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ResourceInfo {
    total_memory_bytes: u64,
    available_memory_bytes: u64,
    available_disk_bytes: u64,
    disk_mount: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DependencyCheck {
    id: &'static str,
    name: &'static str,
    required: bool,
    available: bool,
    detail: String,
    hint: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct EnvironmentSummary {
    ready: bool,
    required_found: usize,
    required_total: usize,
    warnings: Vec<String>,
}

fn first_output_line(command: impl AsRef<OsStr>, args: &[&str]) -> Option<String> {
    let output = Command::new(command).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    stdout
        .lines()
        .chain(stderr.lines())
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(|line| line.chars().take(180).collect())
}

fn ffmpeg_candidates(
    executable: &str,
    resource_dir: Option<&Path>,
) -> Vec<(PathBuf, &'static str)> {
    let mut candidates = Vec::new();

    if let Some(directory) = std::env::var_os("VOICE_TRAINER_FFMPEG_DIR") {
        candidates.push((PathBuf::from(directory).join(executable), "环境变量运行时"));
    }

    #[cfg(debug_assertions)]
    if let Some(project_root) = Path::new(env!("CARGO_MANIFEST_DIR")).parent() {
        if let Some(runtime_root) = runtime_paths::project_runtime_root(project_root) {
            candidates.push((
                runtime_root.join("ffmpeg").join("bin").join(executable),
                "项目本地运行时",
            ));
        }
    }

    if let Some(resource_dir) = resource_dir {
        candidates.push((
            runtime_paths::bundled_runtime_root(resource_dir)
                .join("ffmpeg")
                .join("bin")
                .join(executable),
            "应用内置运行时",
        ));
    }

    if let Some(runtime_root) = runtime_paths::managed_runtime_root() {
        candidates.push((
            runtime_root.join("ffmpeg").join("bin").join(executable),
            "应用托管运行时",
        ));
    }

    #[cfg(all(target_os = "macos", debug_assertions))]
    for directory in ["/opt/homebrew/bin", "/usr/local/bin", "/usr/bin"] {
        candidates.push((PathBuf::from(directory).join(executable), "系统工具目录"));
    }

    #[cfg(debug_assertions)]
    candidates.push((PathBuf::from(executable), "系统 PATH"));
    candidates
}

fn ffmpeg_version(executable: &str, resource_dir: Option<&Path>) -> Option<String> {
    ffmpeg_candidates(executable, resource_dir)
        .into_iter()
        .filter(|(path, source)| *source == "系统 PATH" || path.is_file())
        .find_map(|(path, source)| {
            first_output_line(&path, &["-version"]).map(|version| format!("{version} · {source}"))
        })
}

fn dependency(
    id: &'static str,
    name: &'static str,
    required: bool,
    result: Option<String>,
    hint: &'static str,
) -> DependencyCheck {
    DependencyCheck {
        id,
        name,
        required,
        available: result.is_some(),
        detail: result.unwrap_or_else(|| "未检测到".to_string()),
        hint: hint.to_string(),
    }
}

fn python_candidates(resource_dir: Option<&Path>) -> Vec<(PathBuf, &'static str)> {
    let mut candidates = Vec::new();

    if let Some(executable) = std::env::var_os("VOICE_TRAINER_PYTHON") {
        candidates.push((PathBuf::from(executable), "环境变量运行时"));
    }

    #[cfg(debug_assertions)]
    if let Some(project_root) = Path::new(env!("CARGO_MANIFEST_DIR")).parent() {
        if let Some(runtime_root) = runtime_paths::project_runtime_root(project_root) {
            candidates.push((
                runtime_root.join(runtime_paths::python_relative_path()),
                "项目本地运行时",
            ));
        }
    }

    if let Some(resource_dir) = resource_dir {
        candidates.push((
            runtime_paths::bundled_runtime_root(resource_dir)
                .join(runtime_paths::python_relative_path()),
            "应用内置运行时",
        ));
    }

    if let Some(runtime_root) = runtime_paths::managed_runtime_root() {
        candidates.push((
            runtime_root.join(runtime_paths::python_relative_path()),
            "应用托管运行时",
        ));
    }

    #[cfg(all(target_os = "macos", debug_assertions))]
    for executable in [
        "/opt/homebrew/bin/python3.11",
        "/usr/local/bin/python3.11",
        "/opt/homebrew/bin/python3",
        "/usr/local/bin/python3",
        "/usr/bin/python3",
    ] {
        candidates.push((PathBuf::from(executable), "系统工具目录"));
    }

    #[cfg(debug_assertions)]
    {
        candidates.push((PathBuf::from("python"), "系统 PATH"));
        candidates.push((PathBuf::from("python3"), "系统 PATH"));
    }
    candidates
}

fn python_runtime(resource_dir: Option<&Path>) -> (Option<PathBuf>, Option<String>) {
    python_candidates(resource_dir)
        .into_iter()
        .filter(|(path, source)| *source == "系统 PATH" || path.is_file())
        .find_map(|(path, source)| {
            first_output_line(&path, &["--version"]).and_then(|version| {
                version
                    .starts_with("Python 3.11.")
                    .then(|| (Some(path), Some(format!("{version} · {source}"))))
            })
        })
        .unwrap_or((None, None))
}

fn pytorch_version(python: Option<&Path>) -> Option<String> {
    first_output_line(
        python?,
        &[
            "-c",
            "import torch; print(f'PyTorch {torch.__version__} · CUDA {torch.cuda.is_available()} · MPS {bool(getattr(torch.backends, \"mps\", None) and torch.backends.mps.is_available())}')",
        ],
    )
}

fn accelerator_version(python: Option<&Path>) -> Option<String> {
    if cfg!(target_os = "macos") {
        return first_output_line(
            python?,
            &[
                "-c",
                "import torch; ok=bool(getattr(torch.backends, 'mps', None) and torch.backends.mps.is_available()); print('Apple Metal (MPS)' if ok else '')",
            ],
        )
        .filter(|value| !value.trim().is_empty());
    }
    first_output_line(
        "nvidia-smi",
        &[
            "--query-gpu=name,driver_version,memory.total",
            "--format=csv,noheader",
        ],
    )
}

fn disk_resource(disks: &Disks, current_path: &Path) -> (u64, String) {
    disks
        .iter()
        .filter(|disk| current_path.starts_with(disk.mount_point()))
        .max_by_key(|disk| disk.mount_point().as_os_str().len())
        .or_else(|| disks.iter().max_by_key(|disk| disk.available_space()))
        .map(|disk| {
            (
                disk.available_space(),
                disk.mount_point().to_string_lossy().to_string(),
            )
        })
        .unwrap_or((0, "未知".to_string()))
}

#[tauri::command]
pub async fn check_environment(app: tauri::AppHandle) -> Result<EnvironmentReport, String> {
    let resource_dir = app.path().resource_dir().ok();
    tauri::async_runtime::spawn_blocking(move || build_environment_report(resource_dir))
        .await
        .map_err(|error| format!("环境检测任务异常结束：{error}"))
}

fn build_environment_report(resource_dir: Option<PathBuf>) -> EnvironmentReport {
    let system = System::new_all();
    let disks = Disks::new_with_refreshed_list();
    let current_path = std::env::current_exe()
        .ok()
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_default();
    let (available_disk_bytes, disk_mount) = disk_resource(&disks, &current_path);
    let resource_dir = resource_dir.as_deref();
    let (python_command, python_version) = python_runtime(resource_dir);
    let ffmpeg = runtime_paths::executable_name("ffmpeg");
    let ffprobe = runtime_paths::executable_name("ffprobe");
    let accelerator_name = if cfg!(target_os = "macos") {
        "Apple Metal (MPS)"
    } else {
        "NVIDIA CUDA"
    };

    let dependencies = vec![
        dependency(
            "python",
            "Python",
            true,
            python_version,
            "安装项目锁定的 Python 3.11 运行时",
        ),
        dependency(
            "pytorch",
            "PyTorch",
            true,
            pytorch_version(python_command.as_deref()),
            "在项目 Python 环境中安装锁定版本的 PyTorch",
        ),
        dependency(
            "ffmpeg",
            "FFmpeg",
            true,
            ffmpeg_version(&ffmpeg, resource_dir),
            "将 FFmpeg 放入项目本地或应用托管运行时目录",
        ),
        dependency(
            "ffprobe",
            "FFprobe",
            true,
            ffmpeg_version(&ffprobe, resource_dir),
            "将 FFprobe 与 FFmpeg 放在同一个 bin 目录",
        ),
        dependency(
            "accelerator",
            accelerator_name,
            false,
            accelerator_version(python_command.as_deref()),
            "未检测到时将使用 CPU 兼容模式",
        ),
    ];

    let required_total = dependencies.iter().filter(|item| item.required).count();
    let required_found = dependencies
        .iter()
        .filter(|item| item.required && item.available)
        .count();
    let mut warnings = Vec::new();

    if system.total_memory() < 16 * GIB {
        warnings.push("系统内存低于建议的 16 GB".to_string());
    }
    if available_disk_bytes < 30 * GIB {
        warnings.push("训练磁盘剩余空间低于建议的 30 GB".to_string());
    }
    if !dependencies
        .iter()
        .any(|item| item.id == "accelerator" && item.available)
    {
        warnings.push(format!("未检测到 {accelerator_name}，将使用 CPU 兼容模式"));
    }

    EnvironmentReport {
        checked_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        platform: PlatformInfo {
            os: System::name().unwrap_or_else(|| std::env::consts::OS.to_string()),
            os_version: System::os_version().unwrap_or_else(|| "未知版本".to_string()),
            architecture: System::cpu_arch(),
            host_name: System::host_name().unwrap_or_else(|| "本机".to_string()),
        },
        resources: ResourceInfo {
            total_memory_bytes: system.total_memory(),
            available_memory_bytes: system.available_memory(),
            available_disk_bytes,
            disk_mount,
        },
        summary: EnvironmentSummary {
            ready: required_found == required_total,
            required_found,
            required_total,
            warnings,
        },
        dependencies,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn environment_report_has_consistent_summary() {
        let report = build_environment_report(None);
        let required_total = report
            .dependencies
            .iter()
            .filter(|item| item.required)
            .count();
        let required_found = report
            .dependencies
            .iter()
            .filter(|item| item.required && item.available)
            .count();

        assert_eq!(report.summary.required_total, required_total);
        assert_eq!(report.summary.required_found, required_found);
        assert_eq!(report.summary.ready, required_found == required_total);
        assert!(report.resources.total_memory_bytes > 0);
        assert!(!report.platform.architecture.is_empty());

        let project_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("Tauri crate should have a project root");
        if let Some(runtime_root) = runtime_paths::project_runtime_root(project_root) {
            let local_ffmpeg = runtime_root
                .join("ffmpeg/bin")
                .join(runtime_paths::executable_name("ffmpeg"));
            if local_ffmpeg.is_file() {
                let ffmpeg = report
                    .dependencies
                    .iter()
                    .find(|item| item.id == "ffmpeg")
                    .expect("FFmpeg dependency should exist");
                assert!(ffmpeg.available);
                assert!(ffmpeg.detail.contains("项目本地运行时"));
            }

            let local_python = runtime_root.join(runtime_paths::python_relative_path());
            if local_python.is_file() {
                let python = report
                    .dependencies
                    .iter()
                    .find(|item| item.id == "python")
                    .expect("Python dependency should exist");
                assert!(python.available);
                assert!(python.detail.contains("Python 3.11."));
                assert!(python.detail.contains("项目本地运行时"));
            }
        }
    }
}
