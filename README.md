# Voice Trainer

Voice Trainer 是本地运行的 AI 人声模型训练桌面应用。它负责管理已获授权的人声音频素材、训练 RVC 模型、试听 checkpoint，并在后续阶段导出 Android 端可安装的 `.vcpkg` 模型包。

## 技术栈

- Tauri 2 / Rust
- Vue 3 / TypeScript / Vite
- Element Plus / Pinia / Vue Router
- SQLite（项目元数据持久化）
- Python Worker（JSON Lines 协议，后续方法接入 RVC 训练与 ONNX 导出）

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
runtime-local/macos-arm64/python/bin/python3
runtime-local/macos-arm64/ffmpeg/bin/ffmpeg
runtime-local/macos-arm64/ffmpeg/bin/ffprobe
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

当前 PyTorch 版本由 `runtime-lock/windows-x64-cpu/pytorch-2.7.1-constraints.txt` 固定。RVC v2 源码、Python 依赖及 RMVPE/ContentVec 模型文件分别由 `runtime-lock/rvc-v2` 下的锁文件固定。

准备好 `runtime-local/windows-x64/wheels/rvc-v2` 中的离线 wheels 后，安装 RVC v2 核心依赖：

```powershell
powershell -ExecutionPolicy Bypass -File scripts/install-rvc-runtime.ps1
```

将模型文件放到锁文件指定的运行时路径后，可校验文件大小与 SHA-256：

```powershell
powershell -ExecutionPolicy Bypass -File scripts/verify-rvc-assets.ps1
```

RVC 源码放到 `runtime-local/windows-x64/rvc` 后，可确认 checkout 与锁定提交完全一致：

```powershell
powershell -ExecutionPolicy Bypass -File scripts/verify-rvc-source.ps1
```

## macOS

macOS 同时支持 Apple Silicon (`macos-arm64`) 与 Intel (`macos-x64`) 目录布局。需要预先安装 Xcode Command Line Tools、FFmpeg 和 Python 3.11；应用也会显式检测 Homebrew 的 `/opt/homebrew/bin` 与 `/usr/local/bin`，不依赖从终端继承 `PATH`。

初始化项目 Python 环境：

```bash
bash scripts/setup-python-runtime.sh
```

把匹配当前架构的离线 wheels 分别放入 `runtime-local/macos-*/wheels/pytorch-2.7.1` 和 `runtime-local/macos-*/wheels/rvc-v2` 后执行：

```bash
bash scripts/install-pytorch-runtime.sh
bash scripts/install-rvc-runtime.sh
bash scripts/verify-rvc-assets.sh
bash scripts/verify-rvc-source.sh
```

macOS 的 PyTorch 版本锁位于 `runtime-lock/macos-arm64-cpu` 和 `runtime-lock/macos-x64-cpu`。Apple Silicon 上环境页会检测 Metal/MPS；MPS 不可用时仍可使用 CPU 模式。

桌面端通过 `start_python_worker`、`python_worker_request` 和 `stop_python_worker` 三个 Tauri 命令管理 Worker。协议格式和兼容性规则见 `python-worker/README.md`。

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
runtime-local/windows-x64/python/python.exe -m unittest discover -s python-worker/tests -v
```

### 自包含安装包

正式交付包必须包含 Python、PyTorch、RVC、FFmpeg、FFprobe 和锁定的模型资源，用户无需安装 Homebrew、Python 或其他训练依赖。macOS 的 Apple Silicon 与 Intel 运行时包含不同的原生二进制，需要分别在对应架构机器上构建安装包。

发布运行时中的 Python 必须是可重定位的独立 Python 3.11 发行版，不能使用 `setup-python-runtime.sh` 生成的开发用 venv，也不能是指向 Homebrew 或系统 Python 的符号链接。将完整运行时准备到当前平台的 `runtime-local/<平台>` 后执行：

```bash
npm run tauri:build:bundled
```

该命令会拒绝开发用 venv，并校验独立 Python 3.11、PyTorch 2.7.1、RVC Python 依赖、FFmpeg/FFprobe、锁定的 RVC 源码提交及四个模型文件的大小和 SHA-256。校验通过后才会把运行时嵌入 Tauri 安装包；离线 wheels、Git 元数据和 Python 缓存不会进入最终包。

正式版只使用应用内置或应用托管的运行时，不会依赖用户系统的 Python、FFmpeg 或 shell `PATH`。开发版仍保留系统工具回退，便于本地调试。

## 开发顺序

- [x] 检测 Python、PyTorch、FFmpeg、FFprobe 与本机资源
- [x] 创建、打开、重命名和删除训练项目
- [x] 使用 SQLite 持久化项目元数据
- [x] 项目列表与新建项目界面
- [x] Python Worker JSON Lines 通信协议
- [x] 固定 RVC v2 分支
- [x] 锁定 NumPy、librosa、RMVPE、ContentVec 等训练依赖
- [ ] 音频导入、预处理与训练任务
- [ ] checkpoint 试听与 `.vcpkg` 导出

项目默认完全离线。请只训练本人声音或已经取得明确授权的声音。
