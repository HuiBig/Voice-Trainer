use serde::Serialize;
use std::{
    path::Path,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
use sysinfo::{Disks, System};

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

fn first_output_line(command: &str, args: &[&str]) -> Option<String> {
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

fn python_runtime() -> (Option<&'static str>, Option<String>) {
    for command in ["python", "python3"] {
        if let Some(version) = first_output_line(command, &["--version"]) {
            return (Some(command), Some(version));
        }
    }
    (None, None)
}

fn pytorch_version(python: Option<&str>) -> Option<String> {
    first_output_line(
        python?,
        &[
            "-c",
            "import torch; print(f'PyTorch {torch.__version__} · CUDA {torch.cuda.is_available()}')",
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
pub async fn check_environment() -> Result<EnvironmentReport, String> {
    tauri::async_runtime::spawn_blocking(build_environment_report)
        .await
        .map_err(|error| format!("环境检测任务异常结束：{error}"))
}

fn build_environment_report() -> EnvironmentReport {
    let system = System::new_all();
    let disks = Disks::new_with_refreshed_list();
    let current_path = std::env::current_exe()
        .ok()
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_default();
    let (available_disk_bytes, disk_mount) = disk_resource(&disks, &current_path);
    let (python_command, python_version) = python_runtime();

    let dependencies = vec![
        dependency(
            "python",
            "Python",
            true,
            python_version,
            "安装项目锁定的 Python 3.10/3.11 运行时",
        ),
        dependency(
            "pytorch",
            "PyTorch",
            true,
            pytorch_version(python_command),
            "在项目 Python 环境中安装锁定版本的 PyTorch",
        ),
        dependency(
            "ffmpeg",
            "FFmpeg",
            true,
            first_output_line("ffmpeg", &["-version"]),
            "安装 FFmpeg 并确保 ffmpeg 位于 PATH",
        ),
        dependency(
            "ffprobe",
            "FFprobe",
            true,
            first_output_line("ffprobe", &["-version"]),
            "FFprobe 通常随 FFmpeg 一起安装",
        ),
        dependency(
            "nvidia",
            "NVIDIA CUDA",
            false,
            first_output_line(
                "nvidia-smi",
                &[
                    "--query-gpu=name,driver_version,memory.total",
                    "--format=csv,noheader",
                ],
            ),
            "未检测到时将使用 CPU；后续可配置 NVIDIA CUDA",
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
        .any(|item| item.id == "nvidia" && item.available)
    {
        warnings.push("未检测到 NVIDIA CUDA，将使用 CPU 兼容模式".to_string());
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
        let report = build_environment_report();
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
    }
}
