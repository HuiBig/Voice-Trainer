# Voice Trainer

Voice Trainer 是本地运行的 AI 人声模型训练桌面应用。它负责管理已获授权的人声音频素材、训练 RVC 模型、试听 checkpoint，并在后续阶段导出 Android 端可安装的 `.vcpkg` 模型包。

## 技术栈

- Tauri 2 / Rust
- Vue 3 / TypeScript / Vite
- Element Plus / Pinia / Vue Router
- SQLite（项目元数据持久化）
- Python Worker（下一阶段接入 RVC、RMVPE、ContentVec 与 ONNX）

## 本地开发

```bash
npm install
npm run tauri:dev
```

仅运行 Web 前端：

```bash
npm run dev
```

## 本地训练运行时

开发环境会优先检测以下项目本地运行时，相关大型二进制文件不会提交到 Git：

```text
runtime-local/windows-x64/python/python.exe
runtime-local/windows-x64/ffmpeg/bin/ffmpeg.exe
runtime-local/windows-x64/ffmpeg/bin/ffprobe.exe
```

也可以使用 `VOICE_TRAINER_PYTHON` 指定 Python 可执行文件，或使用 `VOICE_TRAINER_FFMPEG_DIR` 指定包含 FFmpeg 和 FFprobe 的目录。应用随后会依次检测应用内置运行时、应用托管运行时和系统 `PATH`。

准备好官方 Python 3.11.9 embeddable ZIP 后，可初始化便携运行时：

```powershell
powershell -ExecutionPolicy Bypass -File scripts/setup-python-runtime.ps1
```

准备好匹配 Python 3.11 / Windows x64 的 PyTorch wheel 后，可安装 CPU 运行时：

```powershell
powershell -ExecutionPolicy Bypass -File scripts/install-pytorch-runtime.ps1
```

当前 PyTorch 版本由 `runtime-lock/windows-x64-cpu/pytorch-2.7.1-constraints.txt` 固定。训练依赖暂不提前安装，待 RVC v2 分支固定后再统一锁定。

## 项目数据

训练项目元数据保存在系统应用数据目录的 `voice-trainer.sqlite3` 中，包括：

- 项目名称
- 素材目录
- 训练状态
- 创建、更新与最近打开时间

删除项目只会删除 SQLite 中的项目记录，不会删除素材目录中的音频文件。

## 构建与测试

```bash
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo test --manifest-path src-tauri/Cargo.toml
```

## 开发顺序

- [x] 检测 Python、PyTorch、FFmpeg、FFprobe 与本机资源
- [x] 创建、打开、重命名和删除训练项目
- [x] 使用 SQLite 持久化项目元数据
- [x] 项目列表与新建项目界面
- [ ] Python Worker JSON Lines 通信协议
- [ ] 固定 RVC v2 分支
- [ ] 锁定 NumPy、librosa、RMVPE、ContentVec 等训练依赖
- [ ] 音频导入、预处理与训练任务
- [ ] checkpoint 试听与 `.vcpkg` 导出

项目默认完全离线。请只训练本人声音或已经取得明确授权的声音。
