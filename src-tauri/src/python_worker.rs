use crate::runtime_paths;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    sync::{Arc, Mutex},
    thread,
};
use tauri::Manager;

const PROTOCOL_VERSION: u32 = 1;
const MAX_LINE_BYTES: usize = 1024 * 1024;

#[derive(Clone, Default)]
pub struct PythonWorkerState(Arc<Mutex<WorkerManager>>);

#[derive(Default)]
struct WorkerManager {
    process: Option<WorkerProcess>,
    next_id: u64,
}

struct WorkerProcess {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    info: WorkerInfo,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerInfo {
    protocol_version: u32,
    worker_version: String,
    pid: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReadyEvent {
    protocol_version: u32,
    event: String,
    worker_version: String,
    pid: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorkerResponse {
    protocol_version: u32,
    id: Option<String>,
    ok: bool,
    result: Option<Value>,
    error: Option<WorkerError>,
}

#[derive(Debug, Deserialize)]
struct WorkerError {
    code: String,
    message: String,
    details: Option<Value>,
}

fn project_root() -> Option<PathBuf> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
}

fn python_candidates(resource_dir: Option<&Path>) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(executable) = std::env::var_os("VOICE_TRAINER_PYTHON") {
        candidates.push(PathBuf::from(executable));
    }
    #[cfg(debug_assertions)]
    if let Some(root) = project_root() {
        if let Some(runtime_root) = runtime_paths::project_runtime_root(&root) {
            candidates.push(runtime_root.join(runtime_paths::python_relative_path()));
        }
    }
    if let Some(resource_dir) = resource_dir {
        candidates.push(
            runtime_paths::bundled_runtime_root(resource_dir)
                .join(runtime_paths::python_relative_path()),
        );
    }
    if let Some(runtime_root) = runtime_paths::managed_runtime_root() {
        candidates.push(runtime_root.join(runtime_paths::python_relative_path()));
    }
    #[cfg(all(target_os = "macos", debug_assertions))]
    candidates.extend([
        PathBuf::from("/opt/homebrew/bin/python3.11"),
        PathBuf::from("/usr/local/bin/python3.11"),
        PathBuf::from("/opt/homebrew/bin/python3"),
        PathBuf::from("/usr/local/bin/python3"),
        PathBuf::from("/usr/bin/python3"),
    ]);
    #[cfg(debug_assertions)]
    {
        candidates.push(PathBuf::from("python"));
        candidates.push(PathBuf::from("python3"));
    }
    candidates
}

fn worker_script_candidates(resource_dir: Option<&Path>) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(script) = std::env::var_os("VOICE_TRAINER_WORKER") {
        candidates.push(PathBuf::from(script));
    }
    #[cfg(debug_assertions)]
    if let Some(root) = project_root() {
        candidates.push(root.join("python-worker/worker.py"));
    }
    if let Some(resource_dir) = resource_dir {
        candidates.push(resource_dir.join("resources/python-worker/worker.py"));
        candidates.push(resource_dir.join("python-worker/worker.py"));
    }
    candidates
}

fn resolve_python(resource_dir: Option<&Path>) -> Result<PathBuf, String> {
    python_candidates(resource_dir)
        .into_iter()
        .find(|candidate| {
            Command::new(candidate)
                .args([
                    "-c",
                    "import sys; raise SystemExit(0 if sys.version_info[:2] == (3, 11) else 1)",
                ])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .is_ok_and(|status| status.success())
        })
        .ok_or_else(|| "未找到 Python 运行时，请先初始化项目本地运行时".to_string())
}

fn resolve_worker_script(resource_dir: Option<&Path>) -> Result<PathBuf, String> {
    worker_script_candidates(resource_dir)
        .into_iter()
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| "未找到 Python Worker 脚本".to_string())
}

fn read_json_line<T: for<'de> Deserialize<'de>>(reader: &mut impl BufRead) -> Result<T, String> {
    let mut line = String::new();
    let bytes = reader
        .read_line(&mut line)
        .map_err(|error| format!("读取 Python Worker 输出失败：{error}"))?;
    if bytes == 0 {
        return Err("Python Worker 意外关闭了输出流".to_string());
    }
    if bytes > MAX_LINE_BYTES {
        return Err("Python Worker 返回了超过 1 MiB 的协议消息".to_string());
    }
    serde_json::from_str(&line).map_err(|error| format!("Python Worker 返回了无效 JSON：{error}"))
}

impl WorkerProcess {
    fn spawn(resource_dir: Option<&Path>) -> Result<Self, String> {
        let python = resolve_python(resource_dir)?;
        let script = resolve_worker_script(resource_dir)?;
        let mut child = Command::new(&python)
            .arg("-u")
            .arg(&script)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| format!("启动 Python Worker 失败（{}）：{error}", python.display()))?;

        let stdin = child.stdin.take().ok_or("无法连接 Python Worker stdin")?;
        let stdout = child.stdout.take().ok_or("无法连接 Python Worker stdout")?;
        if let Some(stderr) = child.stderr.take() {
            thread::spawn(move || {
                for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                    log::warn!("Python Worker: {line}");
                }
            });
        }

        let mut stdout = BufReader::new(stdout);
        let ready: ReadyEvent = read_json_line(&mut stdout)?;
        if ready.event != "ready" || ready.protocol_version != PROTOCOL_VERSION {
            let _ = child.kill();
            return Err(format!(
                "Python Worker 握手失败：需要协议版本 {PROTOCOL_VERSION}"
            ));
        }
        let info = WorkerInfo {
            protocol_version: ready.protocol_version,
            worker_version: ready.worker_version,
            pid: ready.pid,
        };
        Ok(Self {
            child,
            stdin,
            stdout,
            info,
        })
    }

    fn request(&mut self, id: String, method: &str, params: Value) -> Result<Value, String> {
        let request = json!({
            "protocolVersion": PROTOCOL_VERSION,
            "id": id,
            "method": method,
            "params": params,
        });
        serde_json::to_writer(&mut self.stdin, &request)
            .map_err(|error| format!("序列化 Python Worker 请求失败：{error}"))?;
        self.stdin
            .write_all(b"\n")
            .and_then(|_| self.stdin.flush())
            .map_err(|error| format!("发送 Python Worker 请求失败：{error}"))?;

        let response: WorkerResponse = read_json_line(&mut self.stdout)?;
        if response.protocol_version != PROTOCOL_VERSION || response.id.as_deref() != Some(&id) {
            return Err("Python Worker 响应与请求不匹配".to_string());
        }
        if response.ok {
            return Ok(response.result.unwrap_or(Value::Null));
        }
        let error = response
            .error
            .ok_or("Python Worker 返回了不完整的错误响应")?;
        let details = error
            .details
            .map(|value| format!("：{value}"))
            .unwrap_or_default();
        Err(format!("{}：{}{}", error.code, error.message, details))
    }
}

impl Drop for WorkerProcess {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

impl WorkerManager {
    fn start(&mut self, resource_dir: Option<&Path>) -> Result<WorkerInfo, String> {
        if let Some(process) = &mut self.process {
            if process
                .child
                .try_wait()
                .map_err(|error| error.to_string())?
                .is_none()
            {
                return Ok(process.info.clone());
            }
            self.process = None;
        }
        let process = WorkerProcess::spawn(resource_dir)?;
        let info = process.info.clone();
        self.process = Some(process);
        Ok(info)
    }

    fn request(
        &mut self,
        method: String,
        params: Value,
        resource_dir: Option<&Path>,
    ) -> Result<Value, String> {
        self.start(resource_dir)?;
        self.next_id += 1;
        let id = format!("desktop-{}", self.next_id);
        self.process
            .as_mut()
            .expect("worker was started")
            .request(id, &method, params)
    }

    fn stop(&mut self) -> Result<(), String> {
        if self.process.is_none() {
            return Ok(());
        }
        self.next_id += 1;
        let id = format!("desktop-{}", self.next_id);
        if let Some(process) = &mut self.process {
            process.request(id, "shutdown", json!({}))?;
            process
                .child
                .wait()
                .map_err(|error| format!("等待 Python Worker 退出失败：{error}"))?;
        }
        self.process = None;
        Ok(())
    }
}

fn lock_manager(
    state: &PythonWorkerState,
) -> Result<std::sync::MutexGuard<'_, WorkerManager>, String> {
    state
        .0
        .lock()
        .map_err(|_| "Python Worker 状态锁已损坏".to_string())
}

#[tauri::command]
pub async fn start_python_worker(
    app: tauri::AppHandle,
    state: tauri::State<'_, PythonWorkerState>,
) -> Result<WorkerInfo, String> {
    let state = state.inner().clone();
    let resource_dir = app.path().resource_dir().ok();
    tauri::async_runtime::spawn_blocking(move || {
        lock_manager(&state)?.start(resource_dir.as_deref())
    })
    .await
    .map_err(|error| format!("Python Worker 启动任务异常结束：{error}"))?
}

#[tauri::command]
pub async fn python_worker_request(
    app: tauri::AppHandle,
    state: tauri::State<'_, PythonWorkerState>,
    method: String,
    params: Option<Value>,
) -> Result<Value, String> {
    let state = state.inner().clone();
    let resource_dir = app.path().resource_dir().ok();
    tauri::async_runtime::spawn_blocking(move || {
        lock_manager(&state)?.request(
            method,
            params.unwrap_or_else(|| json!({})),
            resource_dir.as_deref(),
        )
    })
    .await
    .map_err(|error| format!("Python Worker 请求任务异常结束：{error}"))?
}

#[tauri::command]
pub async fn stop_python_worker(state: tauri::State<'_, PythonWorkerState>) -> Result<(), String> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || lock_manager(&state)?.stop())
        .await
        .map_err(|error| format!("Python Worker 停止任务异常结束：{error}"))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_request_uses_expected_envelope() {
        let request = json!({
            "protocolVersion": PROTOCOL_VERSION,
            "id": "test-1",
            "method": "ping",
            "params": {},
        });
        assert_eq!(request["protocolVersion"], 1);
        assert_eq!(request["method"], "ping");
    }

    #[test]
    fn worker_process_round_trip() {
        if resolve_python(None).is_err() {
            eprintln!("skipping worker round trip: Python 3.11 runtime is unavailable");
            return;
        }
        let mut worker = WorkerProcess::spawn(None).expect("worker should start");
        let response = worker
            .request("test-1".to_string(), "ping", json!({"source": "rust"}))
            .expect("ping should succeed");
        assert_eq!(response["pong"], true);
        assert_eq!(response["echo"]["source"], "rust");
        worker
            .request("test-2".to_string(), "shutdown", json!({}))
            .expect("shutdown should succeed");
        assert!(worker.child.wait().expect("worker should exit").success());
    }
}
