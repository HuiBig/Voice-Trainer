# Voice Trainer 开发进度

更新时间：2026-07-15

## 当前阶段

macOS Apple Silicon 自包含训练运行时已完成，下一阶段进入音频素材导入与预处理。

## 已完成

- [x] Tauri / Vue 桌面应用基础框架
- [x] 本机环境、资源及训练依赖检测
- [x] SQLite 训练项目创建、打开、重命名和删除
- [x] Python Worker JSON Lines 通信协议
- [x] RVC v2 源码、Python 依赖和模型资源锁定
- [x] macOS arm64 独立 Python 3.11.14 运行时
- [x] PyTorch 2.7.1、TorchVision 0.22.1 和 TorchAudio 2.7.1
- [x] macOS arm64 FFmpeg 与 FFprobe
- [x] fairseq 0.12.3 与 pyworld 0.3.4 原生 wheel 构建
- [x] RMVPE、ContentVec 和 RVC v2 40k 预训练模型校验
- [x] 正式版只使用应用内置或应用托管运行时
- [x] 自包含运行时构建前完整性检查与 Tauri 资源暂存

## 验证状态

- Rust 测试：8 项通过
- Python 依赖一致性：通过 `pip check`
- RVC 锁定提交：`9f2f0559e6932c10c48642d404e7d2e771d9db43`
- 模型文件大小与 SHA-256：全部通过
- 暂存后的 macOS arm64 运行时：约 1.6 GB

## 进行中

- [ ] 生成并验证自包含 macOS `.app` / `.dmg`
- [ ] 清理重复 wheels 和非运行必需文件
- [ ] 确认 macOS 最低系统版本；当前 SciPy wheel 要求 macOS 14

## 下一里程碑

音频素材导入与扫描：

- 选择或登记项目素材目录
- 扫描支持的音频文件
- 读取时长、采样率、声道和格式
- 标记无法读取或不符合训练要求的素材
- 在项目界面展示素材统计和问题列表

## 后续待完成

- [ ] FFmpeg 转码、切片与静音处理
- [ ] 数据质量分析与训练集预览
- [ ] RVC F0 与 ContentVec 特征提取
- [ ] 训练任务调度、暂停、恢复和日志
- [ ] checkpoint 管理与试听
- [ ] 推理验证和模型质量检查
- [ ] ONNX 与 Android `.vcpkg` 导出
- [ ] macOS Developer ID 签名、公证和干净机器验证
- [ ] macOS Intel 与 Windows 自包含运行时
