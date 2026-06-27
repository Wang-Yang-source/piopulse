# PioPulse 架构漏洞审计报告

生成日期：2026-06-27  
审计范围：`src/main.rs`、`src/app.rs`、`src/worker.rs`、`src/config.rs` 以及主要 UI/worker 交互路径。  
目标：列出会导致刷写错误、状态错乱、数据丢失、资源泄漏或维护成本失控的架构级问题，供 Codex 后续逐项修复。

## 总体判断

PioPulse 当前已经具备串口监控、PlatformIO 自检构建、ESP/probe-rs 刷写、manifest 管理和 TUI 操作能力，但多个关键路径仍以“全局 App 状态 + 后台任务直接发消息 + UI 即时修改持久化文件”的方式组织。这个结构在单设备轻量使用时可运行，在生产线场景、多设备同时插拔、自动刷写、串口监控和构建任务并发时容易出现不可恢复的状态错乱。

优先级最高的问题是：后台任务缺少统一生命周期和取消机制、刷写完成判定依赖全局 channel 状态、配置保存非原子且错误被吞掉、串口/波形缓冲使用 `Vec::remove(0)` 造成高频退化，以及 PlatformIO 临时工程复制/外部命令执行边界不清。

## P0：必须先修

### 1. 后台任务没有统一生命周期管理，退出和切换场景会留下孤儿任务

证据：
- `src/worker.rs:117`、`src/worker.rs:206`、`src/worker.rs:244`、`src/worker.rs:286` 直接 `tokio::spawn` 后不返回 `JoinHandle`。
- `src/worker.rs:328` 串口监控内部又 `spawn_blocking`，外层只等待该 blocking task 完成。
- `src/app.rs:1453`、`src/app.rs:1551`、`src/app.rs:1695` 主要通过删除 sender/cancel sender 影响任务，没有任务注册表。
- `src/main.rs:454` 收到退出信号后设置 `exit = true`，没有集中取消和 join worker。

影响：
- TUI 退出、端口切换、刷写前停止串口监控时，后台 blocking 线程可能仍在读串口或等待外部进程。
- 端口重插后旧任务消息可能写回新 channel，导致状态闪回或错误刷到 UI。
- 自动刷写场景中 probe 任务和 flash 任务可能交错，形成重复启动或过期状态覆盖。

建议修复：
- 新增 `TaskManager` 或 `WorkerRuntime`，集中保存任务句柄、取消 token、任务 generation。
- 每个端口维护 `port_session_id`，worker message 携带 session id；`App::handle_worker_message` 丢弃过期消息。
- 退出时执行 `cancel_all()`，并在合理超时内 `join` 可等待任务。
- PlatformIO build/upload 子进程也必须纳入生命周期，取消时 kill 子进程。

验收标准：
- 启动串口监控后立即退出，进程表中不残留 `piopulse` 子线程导致的串口占用。
- 端口拔插 20 次，旧端口任务消息不会更新新 channel。
- `cargo test` 增加覆盖：过期 `WorkerMessage` 不改变当前端口状态。

### 2. 刷写完成判定错误，批量刷写可能提前结束或永远不结束

证据：
- `src/app.rs:1166` 的 `start_flashing_indices` 只重置被选中的 indices。
- `src/app.rs:1651` 使用 `self.channels.iter().all(|c| c.finished || c.status == "Idle")` 判定“全部完成”。
- 未参与本次刷写但 status 不是 `Idle` 的 channel 会影响本批次；被拔出的 channel 在 `scan_ports` 中直接移除，也会改变完成判定。

影响：
- 批量刷写只选择部分设备时，旧状态可能导致本批次结束条件错误。
- 设备刷写中途拔出时，`is_flashing` 可能无法复位。
- stats 是全局累计值，完成日志里的 Passed/Failed 不是本批次数据。

建议修复：
- 引入 `FlashBatch { id, target_ports, started_at, completed, passed, failed }`。
- `WorkerMessage::Finished` 携带 batch id；只统计当前 batch 的 target。
- 拔出目标端口时将该端口标记为失败或取消，并推进 batch 完成。
- 完成日志使用本批次 counters，不使用累计 stats。

验收标准：
- 单设备、选中设备、全部设备、自动刷写四种路径都使用同一个 batch 完成逻辑。
- 中途拔出目标设备会在 UI 中显示明确失败/取消，并释放 `is_flashing`。

### 3. 配置和 manifest 保存不是原子操作，且多数保存错误被忽略

证据：
- `src/config.rs:145` `save_to_file` 使用 `File::create` 直接覆盖目标文件。
- `src/app.rs:978`、`src/app.rs:997`、`src/app.rs:2636`、`src/app.rs:2824`、`src/app.rs:3460`、`src/app.rs:3488`、`src/app.rs:3530`、`src/app.rs:3651` 多处 `let _ = self.config.save_to_file(...)`。
- `src/main.rs:572` 配置编辑保存同样忽略错误。

影响：
- 程序崩溃、磁盘满或权限错误时可能留下截断的 TOML。
- 用户以为配置已保存，但实际失败，下一次启动回退到旧配置或默认配置。
- `build/piopulse.toml` 在 PlatformIO 自检和 UI 编辑并发时有覆盖风险。

建议修复：
- 实现原子保存：写入同目录临时文件，flush/sync 后 rename。
- 所有保存调用返回 `Result` 并在 UI 日志/通知中明确展示失败。
- 将 manifest 编辑动作集中到 `ManifestService`，避免 UI 事件散落地直接写文件。
- 对 `piopulse.toml` 和 `build/piopulse.toml` 明确所有权：用户配置与生成 manifest 不应互相覆盖。

验收标准：
- 模拟只读目录或无权限路径，UI 显示保存失败且原文件不被截断。
- `cargo test` 覆盖保存失败、原子 rename、manifest 生成不覆盖用户配置。

## P1：高优先级

### 4. App 结构体承担过多职责，UI 状态、业务状态、IO 状态强耦合

证据：
- `src/app.rs:181` 起 `App` 包含 channel、stats、logs、配置、modal、布局、plotter、dashboard、serial、worker tx 等所有状态。
- `src/app.rs:1560` `handle_worker_message` 同时更新生产状态、串口监控、自动回复、波形、图片、PlatformIO 构建结果和音效。
- `src/main.rs:470` 起键盘处理直接操作大量 `App` 字段和保存配置。

影响：
- 新功能容易破坏不相关状态。
- 测试必须构造完整 `App`，导致单元测试成本高。
- UI 输入、业务命令和持久化动作混在一起，错误恢复困难。

建议修复：
- 拆分 domain service：`FlashController`、`SerialController`、`ManifestStore`、`BuildController`、`UiState`。
- `main.rs` 只负责事件循环，将输入事件转换成 `AppCommand`。
- `App::handle_worker_message` 拆为各 controller 的 message reducer。

验收标准：
- 新增刷写状态测试无需创建完整 TUI 布局状态。
- `main.rs` 键盘分发减少为 command dispatch，配置保存不直接出现在事件循环中。

### 5. 串口和波形缓冲在高频数据下性能退化

证据：
- `src/app.rs:1816` 波形历史超过 100 后 `history.remove(0)`。
- `src/app.rs:2044` 串口 timeline 超过 2000 后 `serial_timeline.remove(0)`。
- `src/app.rs:1761` 每个 `SerialData` 都写日志，`format_serial_rx_messages` 可能把高频二进制数据转成大量 UI log。

影响：
- 高频串口数据会造成 O(n) 搬移和 UI 卡顿。
- 二进制流会快速污染日志和 session log。
- 串口监控可能阻塞 worker message channel，进一步影响刷写状态更新。

建议修复：
- 使用 `VecDeque` 或环形缓冲替代 `Vec::remove(0)`。
- 串口日志增加速率限制、二进制摘要模式和最大字节/秒预算。
- `SerialData`、`WaveformData` 使用独立 bounded channel 或采样合并，避免挤占刷写状态消息。

验收标准：
- 115200 和 921600 bps 连续输入 60 秒，TUI 不明显卡顿，内存稳定。
- 二进制流默认不生成逐行日志，只更新摘要和可选 recording。

### 6. PlatformIO 外部命令和临时工程复制边界不清

证据：
- `src/worker.rs:134`、`src/worker.rs:1488` 直接执行 `pio`，依赖 PATH。
- `src/config.rs:1185` 起复制 source tree，但只排除少量目录：`.git`、`.pio`、`build`、`factory`、`target`、`.agents`、`.codex`。
- `src/config.rs:187` `prepare_external_platformio_project` 会把外部工程复制到 temp，再复制 PlatformIO 引用资产。

影响：
- 大型工程可能复制大量无关文件，启动慢、磁盘膨胀。
- 软链接、隐藏目录、生成目录或用户私有文件可能被复制到临时工程。
- `pio` 路径、版本和环境不可控，失败诊断不足。

建议修复：
- 增加 `PlatformIoService`：固定解析 `pio` 路径、版本探测、环境变量白名单、超时和取消。
- 复制策略改为白名单：`src/`、`include/`、`lib/`、`platformio.ini` 和明确引用资产。
- 明确处理 symlink：默认不跟随，或只允许 project root 内部路径。
- 增加临时目录清理策略。

验收标准：
- 外部工程包含大目录、symlink、隐藏目录时不会被无界复制。
- `pio` 不存在时 UI 给出可执行的错误提示。

### 7. sudo 密码验证嵌在 TUI 进程内，安全和体验都较脆弱

证据：
- `src/app.rs:4266` `verify_sudo_password` 通过 `sudo -S -v` 写入密码。
- `src/app.rs:1877` `unlock_admin` 用 sudo 验证作为 Admin Mode 解锁。

影响：
- TUI 自己采集系统密码，风险较高，也容易被日志/崩溃转储误伤。
- 非 Linux、无 sudo、sudo 需要 TTY 或策略变更时功能不可用。
- “管理员模式”与系统 sudo 权限耦合过紧，不利于跨平台发布。

建议修复：
- 将 Admin Mode 改为应用级权限：本地 PIN、配置文件策略或 OS keyring。
- 真正需要提权的操作单独触发系统授权，不把 sudo 密码常驻在 TUI 输入路径。
- 至少增加平台检测和错误提示，不在 Windows/macOS 走同一逻辑。

验收标准：
- Admin Mode 解锁不要求系统 sudo 密码。
- 无 sudo 环境下功能有明确降级路径。

## P2：中优先级

### 8. Flash 后端选择逻辑分散，ESP、probe-rs、PlatformIO upload 的边界不够清晰

证据：
- `src/worker.rs` 同时包含 ESP serial backend、probe-rs backend、PlatformIO uploader、进度解析和命令生成。
- `src/worker.rs:933`、`src/worker.rs:1320` 等生产步骤在不同 backend 中重复实现。

影响：
- 不同后端的行为不一致，例如 NVS、QA、security 状态和 planned bytes 的语义不同。
- 新增芯片或 upload protocol 时容易复制粘贴。

建议修复：
- 定义 `FlashPlan`：包含 images、offsets、nvs、security、qa。
- 后端只执行 plan，不负责生产业务状态。
- 后端选择集中在一个 factory：`EspFlashBackend`、`ProbeRsBackend`、`PlatformIoBackend`。

验收标准：
- 所有后端输出统一的 `FlashEvent` 序列。
- 单元测试可验证同一 config 生成的 FlashPlan。

### 9. manifest 相对路径解析会改变内存模型，保存后可能丢失可移植性

证据：
- `src/config.rs:178` `resolve_relative_paths` 载入时把相对路径转为绝对路径。
- `src/config.rs:145` 保存时直接序列化当前内存状态。

影响：
- 用户打开相对路径 manifest 后保存，会把路径固化成绝对路径。
- `build/piopulse.toml` 包内相对路径和用户级 `piopulse.toml` 的语义容易混淆。

建议修复：
- 区分 `ProjectConfigRaw` 和 `ResolvedProjectConfig`。
- 保存用户编辑时保留原始路径表达；刷写前再 resolve。
- generated factory manifest 可以保持相对路径，但用户配置不要被自动绝对化。

验收标准：
- 载入含相对路径的 TOML，编辑无关字段后保存，路径仍保持相对。

### 10. 日志系统混合 UI ring buffer、session 文件和业务事件

证据：
- `src/app.rs:889` `log` 同时写 session file 和 UI `logs`。
- `src/app.rs:906` `channel_log` 直接拼接 port。
- worker 消息中 `Log` 是字符串，不是结构化事件。

影响：
- 难以过滤、导出或定位某次 batch 的日志。
- 高频串口和构建输出会挤掉重要错误。

建议修复：
- 引入结构化 `AppEvent { level, source, port, batch_id, message }`。
- UI log、文件 log、debug export 分别订阅事件。
- 对 build/serial/flash 设置独立 ring buffer。

验收标准：
- 可按 port/batch/source 过滤日志。
- 高频串口日志不会覆盖刷写失败原因。

## 建议修复顺序

1. 先修 P0-1 和 P0-2：任务生命周期 + FlashBatch。这是后续所有并发问题的根。
2. 再修 P0-3：原子保存和错误展示，避免修复期间继续破坏 manifest。
3. 接着处理 P1-5：缓冲和日志限流，提升长时间运行稳定性。
4. 然后做 P1-4 的拆分，不要一开始大重构；先抽离 `FlashController` 和 `SerialController`。
5. 最后统一 PlatformIO、Admin Mode、后端抽象和路径模型。

## 给 Codex 的执行提示

- 不要一次性重写整个 `App`。先引入小型结构体并迁移单一路径。
- 每次修复都应新增或更新测试，尤其是 batch、保存失败、过期消息、串口缓冲。
- 工作区当前已有未提交修改，修复时必须避免回滚用户改动。
- 优先保持现有 UI 行为不变，只改变状态管理和错误恢复。

## 最小验收命令

```bash
cargo test
cargo clippy --all-targets --all-features
cargo fmt --check
```

如 clippy 因现有代码量过大暂时无法全量通过，至少要对本次修改模块新增定向测试，并在提交说明中列出剩余 clippy 问题。
