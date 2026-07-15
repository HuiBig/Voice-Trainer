use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const AUDIO_EXTENSIONS: &[&str] = &[
    "aac", "aif", "aiff", "flac", "m4a", "mp3", "ogg", "opus", "wav",
];
const MAX_ISSUES: usize = 50;

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MaterialScanReport {
    scanned_at: u64,
    directory: String,
    audio_file_count: u64,
    total_audio_bytes: u64,
    unsupported_file_count: u64,
    issues: Vec<String>,
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn is_audio_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| {
            AUDIO_EXTENSIONS
                .iter()
                .any(|supported| extension.eq_ignore_ascii_case(supported))
        })
        .unwrap_or(false)
}

fn record_issue(issues: &mut Vec<String>, message: String) {
    if issues.len() < MAX_ISSUES {
        issues.push(message);
    }
}

fn scan_directory(root: &Path) -> Result<MaterialScanReport, String> {
    if !root.exists() {
        return Err(format!("素材目录不存在：{}", root.display()));
    }
    if !root.is_dir() {
        return Err(format!("素材路径不是目录：{}", root.display()));
    }

    let mut report = MaterialScanReport {
        scanned_at: now(),
        directory: root.to_string_lossy().to_string(),
        audio_file_count: 0,
        total_audio_bytes: 0,
        unsupported_file_count: 0,
        issues: Vec::new(),
    };
    let mut pending = vec![PathBuf::from(root)];

    while let Some(directory) = pending.pop() {
        let entries = match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(error) => {
                record_issue(
                    &mut report.issues,
                    format!("无法读取目录 {}：{error}", directory.display()),
                );
                continue;
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    record_issue(&mut report.issues, format!("无法读取目录项：{error}"));
                    continue;
                }
            };
            let path = entry.path();
            let file_type = match entry.file_type() {
                Ok(file_type) => file_type,
                Err(error) => {
                    record_issue(
                        &mut report.issues,
                        format!("无法识别 {}：{error}", path.display()),
                    );
                    continue;
                }
            };

            if file_type.is_symlink() {
                continue;
            }
            if file_type.is_dir() {
                pending.push(path);
                continue;
            }
            if !file_type.is_file() {
                continue;
            }
            if !is_audio_file(&path) {
                report.unsupported_file_count += 1;
                continue;
            }

            match entry.metadata() {
                Ok(metadata) => {
                    report.audio_file_count += 1;
                    report.total_audio_bytes =
                        report.total_audio_bytes.saturating_add(metadata.len());
                }
                Err(error) => record_issue(
                    &mut report.issues,
                    format!("无法读取音频文件 {}：{error}", path.display()),
                ),
            }
        }
    }

    Ok(report)
}

#[tauri::command]
pub async fn scan_material_directory(directory: String) -> Result<MaterialScanReport, String> {
    let directory = directory.trim();
    if directory.is_empty() {
        return Err("素材目录不能为空".to_string());
    }
    let path = PathBuf::from(directory);
    tauri::async_runtime::spawn_blocking(move || scan_directory(&path))
        .await
        .map_err(|error| format!("素材扫描任务异常结束：{error}"))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scans_supported_audio_recursively() {
        let root = std::env::temp_dir().join(format!(
            "voice-trainer-material-scan-{}-{}",
            std::process::id(),
            now()
        ));
        let nested = root.join("nested");
        fs::create_dir_all(&nested).unwrap();
        fs::write(root.join("voice.WAV"), b"1234").unwrap();
        fs::write(nested.join("voice.flac"), b"123456").unwrap();
        fs::write(root.join("notes.txt"), b"ignore").unwrap();

        let report = scan_directory(&root).unwrap();
        assert_eq!(report.audio_file_count, 2);
        assert_eq!(report.total_audio_bytes, 10);
        assert_eq!(report.unsupported_file_count, 1);
        assert!(report.issues.is_empty());

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_missing_directory() {
        let path = std::env::temp_dir().join("voice-trainer-directory-that-does-not-exist");
        assert!(scan_directory(&path).unwrap_err().contains("不存在"));
    }
}
