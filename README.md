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

## 构建检查

```bash
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
```

## 当前阶段

- [x] Vue + TypeScript + Tauri 项目初始化
- [x] Element Plus、Pinia 和 Vue Router 接入
- [x] PC 训练端应用外壳与工作台首页
- [ ] 本机训练环境检测
- [ ] 本地项目管理与 SQLite 持久化
- [ ] Python Worker JSON Lines 协议
- [ ] 音频导入与预处理
- [ ] RVC 训练和 `.vcpkg` 导出

项目默认完全离线；请只训练本人声音或已取得明确授权的声音。
