# PioPulse

[![Crates.io](https://img.shields.io/crates/v/piopulse.svg)](https://crates.io/crates/piopulse)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Docs.rs](https://docs.rs/piopulse/badge.svg)](https://docs.rs/piopulse)

默认中文文档。英文版见：[README.en.md](README.en.md)。

**PioPulse** 是一个面向产线的终端界面（TUI）烧录、串口调试和数据可视化工具，主要服务于 ESP32/嵌入式设备的批量生产场景。它不是普通的个人开发板下载器，而是更偏向工厂使用的“多工位智能烧录测试一体机”软件基础。

它的重点是：

- 多设备并发烧录
- 自动识别 USB 串口设备
- 烧录校验与身份写入
- SN、MAC、批次、固件版本等生产追溯
- 串口监视、命令发送、时间线录制和回放解析
- VOFA+ 数据流绘图
- 操作员/管理员权限隔离

## 核心功能

### 多工位烧录

- 自动扫描 USB 串口设备
- 多通道并发烧录
- 每个通道独立显示状态、进度和结果
- 单个设备失败不会阻塞其他设备
- 支持 ESP32 系列串口烧录流程

### 产线烧录信息

烧录页会显示更接近工业烧录器的关键信息：

- 端口
- 目标芯片
- SN
- MAC 地址
- 当前流程
- 进度
- 计划写入字节数
- QA 结果
- 安全状态
- Trace ID

### 生产策略配置

配置页支持维护产线相关参数：

- 校验方式
- 空片检查策略
- 擦除模式
- 增量烧录开关
- NVS 偏移地址
- Secure Boot 策略
- Flash Encryption 策略
- 烧录后锁定策略
- 操作员角色
- 固件版本
- SN 前缀
- 批次号
- MES 地址
- 标签模板
- QA 测试脚本

### 唯一身份写入

当前 ESP32 流程会生成并写入 NVS 数据：

- Serial Number
- Device Name

SN 会结合芯片类型、时间和 MAC 信息生成，也可以通过配置添加前缀。

### 串口终端

界面 1 是串口调试工作台：

- RX/TX 日志
- Hex 接收显示
- Hex 发送
- 自动追加换行
- 波特率切换
- 快捷命令
- 快捷命令面板鼠标滚轮滚动
- 时间线录制
- 回放并重新解析数据

### VOFA+ 绘图

界面 2 支持常见 VOFA+ 数据流解析和绘图：

- FireWater
- JustFloat
- IndexFloat
- 波形视图
- 柱状图
- 直方图
- FFT 频谱
- IMU/图像相关视图

## 运行

```bash
cargo run
```

## 常用快捷键

- `1`：串口终端
- `2`：绘图界面
- `3`：组件仪表盘
- `4`：烧录界面
- `5`：配置界面
- `Space`：在烧录/配置页开始烧录，在串口页进入输入
- `F1`：管理员模式解锁/锁定
- `F2`：显示/隐藏侧边栏
- `Esc`：退出、关闭弹窗或取消编辑

## 项目结构

- `src/main.rs`：终端初始化、事件循环、键盘/鼠标分发
- `src/app.rs`：应用状态、通道状态、统计、端口扫描、交互处理
- `src/ui.rs`：主布局和 Tab 路由
- `src/ui/serial.rs`：串口终端和快捷命令面板
- `src/ui/channels.rs`：批量烧录产线看板
- `src/ui/config.rs`：项目配置和产线参数配置
- `src/worker.rs`：后台烧录任务和串口监视任务
- `src/nvs.rs`：ESP32 NVS 身份数据生成

## 当前状态说明

PioPulse 已经具备 ESP32 串口烧录、MAC 读取、SN/NVS 生成写入、串口监视、数据解析和生产看板能力。

MES 上传、标签打印、Secure Boot eFuse 锁定、Flash Encryption 实际启用、完整硬件 QA 脚本等功能已经进入配置和流程状态，但还需要按具体工厂环境继续接入真实后端或设备接口。
