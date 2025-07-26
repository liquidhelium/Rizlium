# Rizlium 谱面加载流程详解

本文档详细描述了 Rizlium 节奏游戏谱面编辑器从用户点击菜单"打开"到谱面完全加载的完整流程。

## 1. 用户交互 - 菜单点击

**位置**: `/rizlium_editor/src/extensions/game.rs:69-75`

当用户点击菜单中的"打开"选项时：
- 触发系统: `"game.open_dialog"`
- 快捷键: `Ctrl+O`（全局热键）
- 菜单路径: `文件(File) → 打开谱面(Open Chart)`

## 2. 文件对话框打开

**系统**: `open_dialog_and_load_chart()` 位于 `/rizlium_editor/src/extensions/game.rs:126-128`

**调用链**:
1. 调用 `open_dialog()` 函数（位于 `files.rs:35-45`）
2. 使用 `rfd::AsyncFileDialog` 创建跨平台文件选择器
3. 设置文件过滤器，只显示 `.zip` 格式（谱面包格式）
4. 在 `IoTaskPool` 中异步运行，避免阻塞主线程

## 3. 谱面加载启动

**系统**: `EditorCommands::open_dialog_and_load_chart()` 位于 `/rizlium_editor/src/editor_actions.rs:41-46`

**操作**:
- 将文件对话框任务推送到命令队列
- 结果存储在 `PendingDialog` 资源中
- 使用 Bevy 的延迟命令系统确保线程安全

## 4. 文件选择处理

**系统**: `open_chart()` 位于 `/rizlium_editor/src/files.rs:47-58`

**处理流程**:
1. 轮询异步文件对话框任务
2. 用户选择文件后，提取完整文件路径
3. 调用 `editor_command.load_chart(path)` 启动加载
4. 清理 `PendingDialog` 资源

## 5. 加载事件分发

**系统**: `EditorCommands::load_chart()` 位于 `/rizlium_editor/src/editor_actions.rs:34-40`

**执行步骤**:
- 发送 `LoadChartEvent(path)` 到 Bevy 事件系统
- 更新最近文件列表（最多保存4个）
- 将最近文件持久化到磁盘配置文件
- 使用 `Persistent<RecentFiles>` 资源管理

## 6. 谱面加载管道

**系统**: `dispatch_load_event()` 位于 `/rizlium_editor/src/chart_loader.rs:148-167`

**处理逻辑**:
- 接收 `LoadChartEvent` 事件
- 启动异步谱面加载任务
- 将任务句柄存储在 `PendingChart` 资源中
- 处理多个同时加载请求（使用最后一个）

## 7. 异步谱面加载

**函数**: `load_chart()` 位于 `/rizlium_editor/src/chart_loader.rs:98-146`

### 详细步骤:

#### 7.1 文件读取
- 使用 `async_fs::read()` 异步读取 `.zip` 文件
- 处理文件读取错误（权限、不存在等）

#### 7.2 ZIP 解压
- 使用 `ZipArchive` 解析 ZIP 文件结构
- 验证文件完整性

#### 7.3 元数据解析
- 提取并解析 `info.yml` 文件
- 获取谱面名称、格式类型、谱面路径、音乐路径
- 支持两种格式:
  - **Rizline**: 需要格式转换
  - **Rizlium**: 原生格式

#### 7.4 谱面文件加载
- 根据格式类型解析谱面数据
- **Rizline格式**: 先解析为 `RizlineChart`，然后转换为 `Chart`
- **Rizlium格式**: 直接反序列化为 `Chart`

#### 7.5 音频文件处理
- 从 ZIP 中提取音频文件
- 使用 `bevy_kira_audio` 加载为 `StaticSoundData`
- 创建 `AudioSource` 资源供游戏使用

## 8. 谱面解包与设置

**系统**: `unpack_chart()` 位于 `/rizlium_editor/src/chart_loader.rs:169-195`

### 成功处理:
1. **创建项目状态**: 设置为 `ProjectState::Bundle(path, chart)`
2. **音频资源注册**: 将音频添加到 Bevy 的资产系统
3. **创建音频句柄**: 存储为 `GameAudioSource` 资源
4. **发送成功事件**: `ChartLoadingEvent::Success(path)`
5. **日志记录**: 记录加载完成信息

### 失败处理:
- 发送错误事件: `ChartLoadingEvent::Error(err)`
- 提供详细的错误信息给用户

## 9. UI 反馈

**系统**: `report_error_or_add_current()` 位于 `/rizlium_editor/src/files.rs:60-76`

### 成功反馈:
- 显示成功提示: "谱面加载成功"
- 更新 `CurrentChartPath` 资源
- 记录当前谱面路径

### 失败反馈:
- 显示错误提示，包含具体错误原因
- 支持的错误类型:
  - 文件读取失败
  - ZIP解压失败
  - 文件缺失
  - 格式错误
  - 音频转换失败

## 10. 谱面结构

加载完成的谱面包含:

### 10.1 谱面数据 (Chart)
- **音符数据**: 所有可点击的音符信息
- **线条数据**: 轨道线和贝塞尔曲线
- **时间信息**: BPM、时间签名等
- **颜色配置**: 轨道和音符颜色

### 10.2 音频数据
- **音乐文件**: 内嵌在 ZIP 中的音频
- **格式支持**: 支持多种音频格式（通过 Kira 音频引擎）

### 10.3 元数据
- **谱面名称**: 从 info.yml 获取
- **文件路径**: 原始 ZIP 文件路径
- **格式版本**: Rizline 或 Rizlium

## 完整流程图

```
用户点击"打开" → 文件选择对话框 → 选择.zip文件 → 异步加载 → 
ZIP解压 → 解析info.yml → 加载谱面文件 → 加载音频文件 → 
格式转换(如需要) → 创建项目状态 → 注册音频资源 → 
UI更新 → 准备编辑
```

## 技术特点

### 异步处理
- 使用 Bevy 的 `IoTaskPool` 进行异步文件操作
- 非阻塞 UI，保持响应性
- 支持大文件加载

### 错误处理
- 使用 `snafu` crate 提供详细的错误链
- 用户友好的错误提示
- 完整的错误日志记录

### 资源管理
- 使用 Bevy 资源系统管理状态
- 自动内存管理，避免泄漏
- 支持热重载

### 扩展性
- 模块化设计，易于添加新格式
- 插件架构，支持功能扩展
- 事件驱动，便于集成