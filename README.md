# Voice Trainer

Voice Trainer 是个人 AI 歌声转换工具链的 PC 训练端。它负责在本机导入和处理授权人声音频、训练 RVC 模型、试听 checkpoint，并导出 Android 端可安装的 `.vcpkg` 模型包。

## 技术栈

- Tauri 2 / Rust
- Vue 3 / TypeScript / Vite
- Element Plus
- Pinia / Vue Router
- Python Worker（后续接入 RVC、RMVPE、ContentVec 与 ONNX）

## 本地开发

```bash
npm install
npm run tauri:dev
```

仅运行 Web 前端：

```bash
npm run dev
```

### 本地 FFmpeg 运行时

开发环境可将 FFmpeg 放在以下目录，二进制文件不会提交到 Git：

```text
runtime-local/windows-x64/ffmpeg/bin/ffmpeg.exe
runtime-local/windows-x64/ffmpeg/bin/ffprobe.exe
```

也可以通过 `VOICE_TRAINER_FFMPEG_DIR` 指定包含这两个文件的目录。应用检测顺序为：环境变量、项目本地运行时、应用内置运行时、应用托管运行时、系统 `PATH`。

### 本地 Python 运行时

将官方 Python 3.11.9 embeddable ZIP 放到：

```text
runtime-local/windows-x64/downloads/python-3.11.9-embed-amd64.zip
```

然后执行初始化脚本：

```powershell
powershell -ExecutionPolicy Bypass -File scripts/setup-python-runtime.ps1
```

脚本会生成 `runtime-local/windows-x64/python/`，配置 `Lib/site-packages` 搜索路径并运行基础自检。本地 Python、PyTorch wheel 和其他大型运行时文件均不提交到 Git。

应用会优先检测这个项目本地 Python。也可以通过 `VOICE_TRAINER_PYTHON` 指定 `python.exe` 的完整路径；之后依次检查应用内置运行时、应用托管运行时和系统 `PATH`。

准备好 Python 完整安装器和匹配 Python 3.11/Windows x64 的 PyTorch wheel 后，可安装 CPU 运行时依赖：

```powershell
powershell -ExecutionPolicy Bypass -File scripts/install-pytorch-runtime.ps1
```

脚本会在 `runtime-local/windows-x64/python-build/` 建立隔离的构建 Python，并通过 pip 的 `--target` 将 PyTorch 及依赖安装到便携运行时。依赖版本由 `runtime-lock/windows-x64-cpu/pytorch-2.7.1-constraints.txt` 固定。脚本不会修改系统 `PATH`，也不会创建快捷方式或文件关联。

## 构建检查

```bash
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
```

## 当前阶段

- [x] Vue + TypeScript + Tauri 项目初始化
- [x] Element Plus、Pinia 和 Vue Router 接入
- [x] PC 训练端应用外壳与工作台首页
- [x] 本机训练环境检测
- [ ] 本地项目管理与 SQLite 持久化
- [ ] Python Worker JSON Lines 协议
- [ ] 音频导入与预处理
- [ ] RVC 训练和 `.vcpkg` 导出

项目默认完全离线；请只训练本人声音或已取得明确授权的声音。
