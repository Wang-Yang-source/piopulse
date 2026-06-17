pub fn tr<'a>(key: &'a str, lang: &str) -> &'a str {
    let is_zh = lang == "zh";
    match key {
        // Core & Main Modals
        "tab_serial" => {
            if is_zh {
                " [1] 串口 "
            } else {
                " [1] Serial "
            }
        }
        "tab_plot" => {
            if is_zh {
                " [2] 波形 "
            } else {
                " [2] Plot "
            }
        }
        "tab_dash" => {
            if is_zh {
                " [3] 仪表盘 "
            } else {
                " [3] Dash "
            }
        }
        "tab_flash" => {
            if is_zh {
                " [4] 烧录 "
            } else {
                " [4] Flash "
            }
        }
        "tab_settings" => {
            if is_zh {
                " [5] 项目配置 "
            } else {
                " [5] Settings "
            }
        }

        "exit_title" => {
            if is_zh {
                "退出确认"
            } else {
                "EXIT CONFIRMATION"
            }
        }
        "exit_question" => {
            if is_zh {
                "选择以下操作以继续："
            } else {
                "Choose an action below to proceed:"
            }
        }
        "exit_settings" => {
            if is_zh {
                "工具设置"
            } else {
                "Tool Settings"
            }
        }
        "exit_quit" => {
            if is_zh {
                "退出程序"
            } else {
                "Quit App"
            }
        }
        "exit_hint" => {
            if is_zh {
                "Esc/点击外部: 取消 | Tab/方向键: 移动 | Enter: 确认"
            } else {
                "Esc/Click outside: Cancel | Tab/Arrows: Move | Enter: Confirm"
            }
        }

        "auth_title" => {
            if is_zh {
                " 需要管理员授权 "
            } else {
                " Admin Authorization Required "
            }
        }
        "auth_msg" => {
            if is_zh {
                "输入系统 sudo 密码进行授权："
            } else {
                "Enter system sudo password to authorize:"
            }
        }
        "auth_cancel" => {
            if is_zh {
                "按 Enter 提交 | 点击外部 / Esc 取消"
            } else {
                "Press Enter to submit | Click outside / Esc to cancel"
            }
        }
        "auth_error" => {
            if is_zh {
                "密码错误。按 Enter 重试。"
            } else {
                "Incorrect password. Press Enter to retry."
            }
        }

        "f1_toggle_admin" => {
            if is_zh {
                "：切换管理员模式 | "
            } else {
                ": Toggle Admin | "
            }
        }
        "f2_toggle_sidebar" => {
            if is_zh {
                "：切换侧边栏 | "
            } else {
                ": Toggle Sidebar | "
            }
        }
        "space_hint_serial" => {
            if is_zh {
                "进入打字模式"
            } else {
                "Type Space"
            }
        }
        "space_hint_flash" => {
            if is_zh {
                "烧录选中"
            } else {
                "Flash Selected"
            }
        }
        "space_hint_focus" => {
            if is_zh {
                "聚焦"
            } else {
                "Focus"
            }
        }
        "space_hint" => {
            if is_zh {
                "：{} | "
            } else {
                ": {} | "
            }
        }
        "add_module" => {
            if is_zh {
                "：添加模块 | "
            } else {
                ": Add Module | "
            }
        }
        "delete_module" => {
            if is_zh {
                "：删除模块 | "
            } else {
                ": Delete Module | "
            }
        }
        "clear_stats" => {
            if is_zh {
                "：清空统计 | "
            } else {
                ": Clear Stats | "
            }
        }
        "tabs_nav" => {
            if is_zh {
                "：切换标签页 | "
            } else {
                ": Tabs | "
            }
        }
        "menu_hint" => {
            if is_zh {
                "：菜单"
            } else {
                ": Menu"
            }
        }

        "tool_settings_title" => {
            if is_zh {
                " 工具设置 "
            } else {
                " TOOL SETTINGS "
            }
        }
        "tool_settings_question" => {
            if is_zh {
                "选择语言 / Select Language:"
            } else {
                "Select Language / 选择语言:"
            }
        }
        "tool_settings_en" => {
            if is_zh {
                "English (英文)"
            } else {
                "English"
            }
        }
        "tool_settings_zh" => {
            if is_zh {
                "简体中文 (Chinese)"
            } else {
                "简体中文"
            }
        }
        "tool_settings_hint" => {
            if is_zh {
                "Esc: 取消 | Tab/方向键: 移动 | Enter: 保存"
            } else {
                "Esc: Cancel | Tab/Arrows: Move | Enter: Save"
            }
        }
        "port_menu_title" => {
            if is_zh {
                "选择串口端口"
            } else {
                "SELECT SERIAL PORT"
            }
        }
        "port_menu_hint" => {
            if is_zh {
                "Esc/点击外部: 取消 | ↑/↓: 移动 | Enter: 确认"
            } else {
                "Esc/Click outside: Cancel | ↑/↓: Move | Enter: Confirm"
            }
        }

        "admin_mode_header" => {
            if is_zh {
                " [管理员模式] "
            } else {
                " [ADMIN MODE] "
            }
        }
        "operator_mode_header" => {
            if is_zh {
                " [操作员模式] "
            } else {
                " [OPERATOR MODE] "
            }
        }

        // Serial Tab (serial.rs)
        "serial_rx_title" => {
            if is_zh {
                " 串口控制台接收器 [{}] "
            } else {
                " Serial Console Receiver [{}] "
            }
        }
        "serial_typing_mode" => {
            if is_zh {
                " 打字模式... (按 [Esc] 退出，按 [Enter] 发送) "
            } else {
                " Typing Mode... (Press [Esc] to exit, [Enter] to send) "
            }
        }
        "serial_send_console" => {
            if is_zh {
                " 数据发送控制台 (按 [i] 或 [Enter] 开始输入) "
            } else {
                " Send Data Console (Press [i] or [Enter] to type) "
            }
        }
        "serial_placeholder" => {
            if is_zh {
                "在此处输入命令..."
            } else {
                "Type command here..."
            }
        }
        "serial_port_info" => {
            if is_zh {
                " 端口信息 "
            } else {
                " Port Info "
            }
        }
        "serial_port" => {
            if is_zh {
                "端口: "
            } else {
                "Port: "
            }
        }
        "serial_baud" => {
            if is_zh {
                "波特率 [b]: "
            } else {
                "Baud [b]: "
            }
        }
        "serial_bits" => {
            if is_zh {
                "数据位: "
            } else {
                "Bits: "
            }
        }
        "serial_options_title" => {
            if is_zh {
                " 选项 "
            } else {
                " Options "
            }
        }
        "serial_auto_scroll" => {
            if is_zh {
                "自动滚动 "
            } else {
                "Auto Scroll "
            }
        }
        "serial_send_newline" => {
            if is_zh {
                "发送换行 "
            } else {
                "Send Newline "
            }
        }
        "serial_hex_display" => {
            if is_zh {
                "Hex 显示  "
            } else {
                "Hex Display  "
            }
        }
        "serial_hex_sending" => {
            if is_zh {
                "Hex 发送  "
            } else {
                "Hex Sending  "
            }
        }
        "serial_quick_commands" => {
            if is_zh {
                " 快捷命令 "
            } else {
                " Quick Commands "
            }
        }
        "serial_cmd" => {
            if is_zh {
                "命令"
            } else {
                "Cmd"
            }
        }
        "serial_desc" => {
            if is_zh {
                "描述"
            } else {
                "Description"
            }
        }
        "serial_ping_desc" => {
            if is_zh {
                "测试 Ping"
            } else {
                "Test Ping"
            }
        }
        "serial_version_desc" => {
            if is_zh {
                "获取版本"
            } else {
                "Get Version"
            }
        }
        "serial_product_desc" => {
            if is_zh {
                "获取产品标识信息"
            } else {
                "Get Product Info"
            }
        }
        "serial_echo_on_desc" => {
            if is_zh {
                "开启指令回显"
            } else {
                "Enable Echo"
            }
        }
        "serial_echo_off_desc" => {
            if is_zh {
                "关闭指令回显"
            } else {
                "Disable Echo"
            }
        }
        "serial_signal_desc" => {
            if is_zh {
                "查询信号质量"
            } else {
                "Query Signal Quality"
            }
        }
        "serial_ip_desc" => {
            if is_zh {
                "查询本地 IP 地址"
            } else {
                "Query Local IP"
            }
        }
        "serial_wifi_mode_desc" => {
            if is_zh {
                "查询 Wi-Fi 模式"
            } else {
                "Query Wi-Fi Mode"
            }
        }
        "serial_uart_desc" => {
            if is_zh {
                "查询当前串口配置"
            } else {
                "Query UART Config"
            }
        }
        "serial_reset_defaults_desc" => {
            if is_zh {
                "恢复出厂默认配置"
            } else {
                "Restore Default Config"
            }
        }
        "serial_reboot_desc" => {
            if is_zh {
                "重启开发板"
            } else {
                "Reboot Board"
            }
        }
        "serial_list_desc" => {
            if is_zh {
                "列出选项"
            } else {
                "List Options"
            }
        }

        // Plotter Tab (plotter.rs)
        "plot_title" => {
            if is_zh {
                " 波形绘制器 "
            } else {
                " WAVEFORM PLOTTER "
            }
        }
        "plot_port_hint" => {
            if is_zh {
                " 选择端口   "
            } else {
                " select port   "
            }
        }
        "plot_protocol_hint" => {
            if is_zh {
                " 协议   "
            } else {
                " protocol   "
            }
        }
        "plot_view_hint" => {
            if is_zh {
                " 视图   "
            } else {
                " view   "
            }
        }
        "plot_start_hint" => {
            if is_zh {
                " 启动/暂停   "
            } else {
                " start/pause   "
            }
        }
        "plot_clear_hint" => {
            if is_zh {
                " 清空缓冲区"
            } else {
                " clear buffer"
            }
        }
        "plot_latest" => {
            if is_zh {
                "(最新)"
            } else {
                "(Latest)"
            }
        }
        "plot_overview" => {
            if is_zh {
                "采集概览"
            } else {
                "Acquisition"
            }
        }
        "plot_samples" => {
            if is_zh {
                "样本"
            } else {
                "Samples"
            }
        }
        "plot_channels" => {
            if is_zh {
                "通道"
            } else {
                "Channels"
            }
        }
        "plot_range" => {
            if is_zh {
                "范围"
            } else {
                "Range"
            }
        }
        "plot_trigger_ref" => {
            if is_zh {
                "触发"
            } else {
                "Trigger"
            }
        }
        "plot_signal_locked" => {
            if is_zh {
                "信号锁定"
            } else {
                "Signal Locked"
            }
        }
        "plot_signal_waiting" => {
            if is_zh {
                "等待信号"
            } else {
                "Waiting Signal"
            }
        }
        "plot_scope_title" => {
            if is_zh {
                " 波形观测器Scope ({}){} "
            } else {
                " Waveform Scope ({}){} "
            }
        }
        "plot_barchart_title" => {
            if is_zh {
                " 柱状图 - 通道实时数值 ({}){} "
            } else {
                " Bar Chart - Latest Channel Values ({}){} "
            }
        }
        "plot_rx_console" => {
            if is_zh {
                "接收控制台"
            } else {
                "RX Console"
            }
        }
        "plot_no_data" => {
            if is_zh {
                "暂无串口数据。请选择端口并开始流传输。"
            } else {
                "No serial data yet. Select a port and start streaming."
            }
        }
        "plot_lines" => {
            if is_zh {
                " 行"
            } else {
                " lines"
            }
        }
        "plot_port" => {
            if is_zh {
                "端口 "
            } else {
                "Port "
            }
        }
        "plot_rx_parsed_title" => {
            if is_zh {
                " 接收 / 解析数据 "
            } else {
                " Receive / Parsed Data "
            }
        }
        "plot_no_ports" => {
            if is_zh {
                "无可用端口"
            } else {
                "No ports available"
            }
        }
        "plot_sim_on" => {
            if is_zh {
                "  模拟开启"
            } else {
                "  SIM ON"
            }
        }
        "plot_sim_off" => {
            if is_zh {
                "  模拟关闭"
            } else {
                "  SIM OFF"
            }
        }
        "plot_active" => {
            if is_zh {
                "  激活"
            } else {
                "  ACTIVE"
            }
        }
        "plot_ports_list" => {
            if is_zh {
                " 端口列表 "
            } else {
                " Ports "
            }
        }
        "plot_state" => {
            if is_zh {
                "状态     "
            } else {
                "State     "
            }
        }
        "plot_connected" => {
            if is_zh {
                "已连接"
            } else {
                "Connected"
            }
        }
        "plot_paused" => {
            if is_zh {
                "已暂停"
            } else {
                "Paused"
            }
        }
        "plot_baud" => {
            if is_zh {
                "波特率    "
            } else {
                "Baud      "
            }
        }
        "plot_format" => {
            if is_zh {
                "格式      "
            } else {
                "Format    "
            }
        }
        "plot_flow" => {
            if is_zh {
                "流控      "
            } else {
                "Flow      "
            }
        }
        "plot_none" => {
            if is_zh {
                "无"
            } else {
                "None"
            }
        }
        "plot_line_end" => {
            if is_zh {
                "换行符    "
            } else {
                "Line end  "
            }
        }
        "plot_buffer" => {
            if is_zh {
                "缓冲区    "
            } else {
                "Buffer    "
            }
        }
        "plot_profile_title" => {
            if is_zh {
                " 串口配置 "
            } else {
                " Serial Profile "
            }
        }
        "plot_waiting_data" => {
            if is_zh {
                "正在等待串口数据流..."
            } else {
                "Waiting for serial stream data..."
            }
        }
        "plot_live_stats" => {
            if is_zh {
                " 实时统计数据 "
            } else {
                " Live Statistics "
            }
        }
        "plot_tx_state_ready" => {
            if is_zh {
                "就绪"
            } else {
                "Ready"
            }
        }
        "plot_tx_state_inactive" => {
            if is_zh {
                "未激活"
            } else {
                "Inactive"
            }
        }
        "plot_tx_mode" => {
            if is_zh {
                "模式 "
            } else {
                "Mode "
            }
        }
        "plot_tx_eol" => {
            if is_zh {
                "结束符 "
            } else {
                "EOL "
            }
        }
        "plot_tx_placeholder" => {
            if is_zh {
                "待发送端口配置就绪后在此输入命令"
            } else {
                "type command here after TX input is wired"
            }
        }
        "plot_tx_enter_send" => {
            if is_zh {
                "[按 Enter 发送]"
            } else {
                "[Enter Send]"
            }
        }
        "plot_tx_history_hint" => {
            if is_zh {
                "[按上下键切换历史记录]"
            } else {
                "[Up/Down History]"
            }
        }
        "plot_tx_quick" => {
            if is_zh {
                "快捷命令: "
            } else {
                "Quick: "
            }
        }
        "plot_tx_send_title" => {
            if is_zh {
                " 发送命令 "
            } else {
                " Send Command "
            }
        }

        // Channels / Flasher Tab (channels.rs)
        "flash_engine_title" => {
            if is_zh {
                " 批量烧录引擎 "
            } else {
                " Batch Flasher Engine "
            }
        }
        "flash_no_devices" => {
            if is_zh {
                "   未检测到活动的烧录设备。"
            } else {
                "   No active flashing devices detected."
            }
        }
        "flash_target_chip" => {
            if is_zh {
                "   目标芯片:  "
            } else {
                "   Target Chip:  "
            }
        }
        "flash_baud_rate" => {
            if is_zh {
                "   波特率:  "
            } else {
                "   Baud Rate:  "
            }
        }
        "flash_mode" => {
            if is_zh {
                "   烧录模式:   "
            } else {
                "   Flash Mode:   "
            }
        }
        "flash_size" => {
            if is_zh {
                "   闪存大小: "
            } else {
                "   Flash Size: "
            }
        }
        "flash_connect_usb" => {
            if is_zh {
                "   连接 USB 设备以自动扫描并开始烧录..."
            } else {
                "   Connect USB devices to auto-scan and begin flashing..."
            }
        }
        "flash_hint" => {
            if is_zh {
                "   提示: 按 "
            } else {
                "   Hint: Press "
            }
        }
        "flash_spacebar" => {
            if is_zh {
                "空格键"
            } else {
                "Spacebar"
            }
        }
        "flash_rescan" => {
            if is_zh {
                " 手动重新扫描端口。"
            } else {
                " to manually re-scan ports."
            }
        }
        "flash_dashboard_title" => {
            if is_zh {
                " 批量烧录监控仪表盘 "
            } else {
                " BATCH FLASHING MONITOR DASHBOARD "
            }
        }
        "flash_devices_count" => {
            if is_zh {
                "  端口总数: "
            } else {
                "  Total Ports: "
            }
        }
        "flash_idle_count" => {
            if is_zh {
                "  |  空闲: "
            } else {
                "  |  Idle: "
            }
        }
        "flash_flashing_count" => {
            if is_zh {
                "  |  烧录中: "
            } else {
                "  |  Flashing: "
            }
        }
        "flash_success_count" => {
            if is_zh {
                "  |  当前成功: "
            } else {
                "  |  Current Pass: "
            }
        }
        "flash_failed_count" => {
            if is_zh {
                "  |  当前失败: "
            } else {
                "  |  Current Fail: "
            }
        }
        "flash_detecting" => {
            if is_zh {
                "检测中..."
            } else {
                "Detecting..."
            }
        }
        "flash_status_success" => {
            if is_zh {
                "成功"
            } else {
                "SUCCESS"
            }
        }
        "flash_status_failed" => {
            if is_zh {
                "失败 ({})"
            } else {
                "FAILED ({})"
            }
        }
        "flash_status_idle" => {
            if is_zh {
                "空闲"
            } else {
                "Idle"
            }
        }
        "flash_devices_title" => {
            if is_zh {
                " 批量烧录设备列表 "
            } else {
                " Batch Flasher Devices "
            }
        }

        // Config Tab (config.rs)
        "config_title" => {
            if is_zh {
                " 配置项目 "
            } else {
                " Configuration "
            }
        }
        "config_proj_name" => {
            if is_zh {
                "项目名称:"
            } else {
                "Project Name:"
            }
        }
        "config_chip_type" => {
            if is_zh {
                "芯片类型:"
            } else {
                "Chip Type:"
            }
        }
        "config_baud_rate" => {
            if is_zh {
                "波特率:"
            } else {
                "Baud Rate:"
            }
        }
        "config_flash_mode" => {
            if is_zh {
                "烧录模式:"
            } else {
                "Flash Mode:"
            }
        }
        "config_flash_freq" => {
            if is_zh {
                "闪存频率:"
            } else {
                "Flash Freq:"
            }
        }
        "config_flash_size" => {
            if is_zh {
                "闪存大小:"
            } else {
                "Flash Size:"
            }
        }
        "config_bootloader_offset" => {
            if is_zh {
                "Bootloader 偏移量:"
            } else {
                "Bootloader Offset:"
            }
        }
        "config_bootloader_path" => {
            if is_zh {
                "Bootloader 路径:"
            } else {
                "Bootloader Path:"
            }
        }
        "config_partitions_offset" => {
            if is_zh {
                "分区表偏移量:"
            } else {
                "Partitions Offset:"
            }
        }
        "config_partitions_path" => {
            if is_zh {
                "分区表路径:"
            } else {
                "Partitions Path:"
            }
        }
        "config_otadata_offset" => {
            if is_zh {
                "OTA 数据偏移量:"
            } else {
                "OTA Data Offset:"
            }
        }
        "config_otadata_path" => {
            if is_zh {
                "OTA 数据路径:"
            } else {
                "OTA Data Path:"
            }
        }
        "config_app_offset" => {
            if is_zh {
                "应用程序偏移量:"
            } else {
                "App Offset:"
            }
        }
        "config_app_path" => {
            if is_zh {
                "应用程序路径:"
            } else {
                "App Path:"
            }
        }
        "config_nvs_offset" => {
            if is_zh {
                "NVS 偏移量:"
            } else {
                "NVS Offset:"
            }
        }
        "config_verify_method" => {
            if is_zh {
                "校验方式:"
            } else {
                "Verify Method:"
            }
        }
        "config_blank_check" => {
            if is_zh {
                "空片检查:"
            } else {
                "Blank Check:"
            }
        }
        "config_erase_mode" => {
            if is_zh {
                "擦除模式:"
            } else {
                "Erase Mode:"
            }
        }
        "config_incremental_programming" => {
            if is_zh {
                "增量烧录:"
            } else {
                "Incremental:"
            }
        }
        "config_secure_boot" => {
            if is_zh {
                "Secure Boot:"
            } else {
                "Secure Boot:"
            }
        }
        "config_flash_encryption" => {
            if is_zh {
                "Flash 加密:"
            } else {
                "Flash Encryption:"
            }
        }
        "config_lock_after_flash" => {
            if is_zh {
                "烧录后锁定:"
            } else {
                "Lock After Flash:"
            }
        }
        "config_operator_role" => {
            if is_zh {
                "操作员角色:"
            } else {
                "Operator Role:"
            }
        }
        "config_firmware_version" => {
            if is_zh {
                "固件版本:"
            } else {
                "Firmware Version:"
            }
        }
        "config_sn_prefix" => {
            if is_zh {
                "SN 前缀:"
            } else {
                "SN Prefix:"
            }
        }
        "config_lot_code" => {
            if is_zh {
                "批次号:"
            } else {
                "Lot Code:"
            }
        }
        "config_mes_endpoint" => {
            if is_zh {
                "MES 地址:"
            } else {
                "MES Endpoint:"
            }
        }
        "config_label_template" => {
            if is_zh {
                "标签模板:"
            } else {
                "Label Template:"
            }
        }
        "config_qa_test_script" => {
            if is_zh {
                "QA 测试脚本:"
            } else {
                "QA Test Script:"
            }
        }
        "config_do_not_chg_bin" => {
            if is_zh {
                "禁止修改Bin头:"
            } else {
                "Do Not Chg Bin:"
            }
        }
        "config_inspector_title" => {
            if is_zh {
                " 检查器 "
            } else {
                " Inspector "
            }
        }
        "config_status_editing" => {
            if is_zh {
                "编辑中"
            } else {
                "EDITING"
            }
        }
        "config_status_unlocked" => {
            if is_zh {
                "已解锁"
            } else {
                "UNLOCKED"
            }
        }
        "config_status_locked" => {
            if is_zh {
                "已锁定 (只读)"
            } else {
                "LOCKED (READ-ONLY)"
            }
        }
        "config_parameter" => {
            if is_zh {
                "  参数: "
            } else {
                "  Parameter: "
            }
        }
        "config_status" => {
            if is_zh {
                "  ·  状态: "
            } else {
                "  ·  Status: "
            }
        }
        "config_value" => {
            if is_zh {
                "  数值: "
            } else {
                "  Value: "
            }
        }
        "config_guide" => {
            if is_zh {
                "  指南: "
            } else {
                "  Guide: "
            }
        }
        "config_guide_locked" => {
            if is_zh {
                "按 F1 解锁。使用上/下方向键或点击以选择字段。"
            } else {
                "Press F1 to unlock. Use Up/Down Arrows or click to select fields."
            }
        }
        "config_guide_editing" => {
            if is_zh {
                "键入新值。按 Enter 保存，按 Esc 取消。"
            } else {
                "Type new value. Press Enter to save, Esc to cancel."
            }
        }
        "config_guide_unlocked" => {
            if is_zh {
                "按 Enter 或点击以编辑。按 F1 锁定。"
            } else {
                "Press Enter or click to edit. Press F1 to lock."
            }
        }

        // Sidebar Panel (sidebar.rs)
        "sidebar_stats_title" => {
            if is_zh {
                " 累计生产数据 "
            } else {
                " Cumulative Stats "
            }
        }
        "sidebar_total_attempted" => {
            if is_zh {
                "累计尝试: "
            } else {
                "Total Attempted: "
            }
        }
        "sidebar_passed" => {
            if is_zh {
                "累计成功 (OK):     "
            } else {
                "Passed (OK):     "
            }
        }
        "sidebar_failed" => {
            if is_zh {
                "累计失败 (FAIL):   "
            } else {
                "Failed (FAIL):   "
            }
        }
        "sidebar_yield_rate" => {
            if is_zh {
                "累计良率:          "
            } else {
                "Yield Rate:      "
            }
        }
        "sidebar_elapsed_time" => {
            if is_zh {
                "运行时间:        "
            } else {
                "Elapsed Time:    "
            }
        }
        "sidebar_monitor_title" => {
            if is_zh {
                " 串口监视器 "
            } else {
                " Serial Monitor "
            }
        }

        // Dashboard / Widgets Tab (widgets/mod.rs)
        "dash_no_modules" => {
            if is_zh {
                "     当前仪表盘未加载任何模块。"
            } else {
                "     No modules are currently loaded in this dashboard."
            }
        }
        "dash_press_a" => {
            if is_zh {
                "     按 [A] "
            } else {
                "     Press [A] "
            }
        }
        "dash_open_catalog" => {
            if is_zh {
                "打开 Ratatui 模块目录并添加组件。"
            } else {
                "to open the Ratatui module catalog and add items."
            }
        }
        "dash_press_d" => {
            if is_zh {
                "     按 [D] "
            } else {
                "     Press [D] "
            }
        }
        "dash_remove_pane" => {
            if is_zh {
                "移除/删除当前选中的分屏。"
            } else {
                "to remove/delete the currently focused pane."
            }
        }
        "dash_press_tab" => {
            if is_zh {
                "     按 [Tab] / 左右方向键 "
            } else {
                "     Press [Tab] / Left-Right Arrows "
            }
        }
        "dash_navigate_panes" => {
            if is_zh {
                "切换选中的分屏。"
            } else {
                "to navigate focused panes."
            }
        }
        "dash_split_hint" => {
            if is_zh {
                "     💡 提示：分屏会在水平或垂直方向上自动分割！"
            } else {
                "     💡 Hint: Panes will auto-split horizontally & vertically!"
            }
        }
        "dash_tiling_workspace" => {
            if is_zh {
                " 平铺工作区 "
            } else {
                " Tiling Workspace "
            }
        }

        "dash_select_module_title" => {
            if is_zh {
                "选择要添加到分屏的 Ratatui 模块"
            } else {
                "Select Ratatui Module to Add to Pane"
            }
        }
        "dash_search" => {
            if is_zh {
                " 搜索: "
            } else {
                " Search: "
            }
        }
        "dash_modal_hint" => {
            if is_zh {
                " 按 [↑/↓] 导航 | [ENTER] 添加 | [ESC] 关闭"
            } else {
                " Press [↑/↓] to navigate | [ENTER] to add | [ESC] to close"
            }
        }
        "dash_catalog_title" => {
            if is_zh {
                " Ratatui 模块目录 "
            } else {
                " Ratatui Module Catalog "
            }
        }

        // Widget Catalog Item Descriptions
        "widget_button_desc" => {
            if is_zh {
                "Ratatui TUI 按钮面板"
            } else {
                "Ratatui TUI Button Panel"
            }
        }
        "widget_cube_desc" => {
            if is_zh {
                "Ratatui 画布 3D 旋转立方体"
            } else {
                "Ratatui Canvas Orientation Cube"
            }
        }
        "widget_dashboard_desc" => {
            if is_zh {
                "Ratatui TUI 系统仪表盘"
            } else {
                "Ratatui TUI System Dashboard"
            }
        }
        "widget_delay_desc" => {
            if is_zh {
                "Ratatui TUI 延时触发器"
            } else {
                "Ratatui TUI Delayed Trigger"
            }
        }
        "widget_dial_desc" => {
            if is_zh {
                "Ratatui TUI 刻度盘面板"
            } else {
                "Ratatui TUI Dial Panel"
            }
        }
        "widget_example_desc" => {
            if is_zh {
                "Rust/Ratatui 模块示例模板"
            } else {
                "Example Rust/Ratatui Module Template"
            }
        }
        "widget_gauge_desc" => {
            if is_zh {
                "Ratatui 进度条/遥测仪表"
            } else {
                "Ratatui Gauge Telemetry Meter"
            }
        }
        "widget_image_desc" => {
            if is_zh {
                "Ratatui 画布 图像/感兴趣区域预览"
            } else {
                "Ratatui Canvas Image/ROI Preview"
            }
        }
        "widget_joystick_desc" => {
            if is_zh {
                "Ratatui 画布 摇杆网格"
            } else {
                "Ratatui Canvas Joystick Grid"
            }
        }
        "widget_knob_desc" => {
            if is_zh {
                "Ratatui TUI 精准旋钮"
            } else {
                "Ratatui TUI Precision Knob"
            }
        }
        "widget_light_desc" => {
            if is_zh {
                "Ratatui TUI 状态指示灯"
            } else {
                "Ratatui TUI Status Lights"
            }
        }
        "widget_pad_desc" => {
            if is_zh {
                "Ratatui 画布 双轴触摸板"
            } else {
                "Ratatui Canvas Dual-Axis Pad"
            }
        }
        "widget_ring_desc" => {
            if is_zh {
                "Ratatui TUI 环形仪表盘"
            } else {
                "Ratatui TUI Ring Dial"
            }
        }
        "widget_slider_desc" => {
            if is_zh {
                "Ratatui TUI 参数滑块"
            } else {
                "Ratatui TUI Parameter Slider"
            }
        }
        "widget_toggle_desc" => {
            if is_zh {
                "Ratatui TUI 锁存开关"
            } else {
                "Ratatui TUI Latched Switch"
            }
        }

        // Individual Widget Titles
        "widget_button_title" => {
            if is_zh {
                "按钮：Ratatui 按钮面板"
            } else {
                "button: Ratatui Button Panel"
            }
        }
        "widget_cube_title" => {
            if is_zh {
                "立方体：3D 姿态"
            } else {
                "cube: 3D Orientation"
            }
        }
        "widget_cube_title_manual" => {
            if is_zh {
                "立方体：3D 姿态 (T: 手动模式, UJIKOL/方向键: 控制)"
            } else {
                "cube: 3D Orientation (T: Manual Mode, UJIKOL/Arrows: Ctrl)"
            }
        }
        "widget_dashboard_title" => {
            if is_zh {
                "仪表盘：Ratatui 电机诊断"
            } else {
                "dashboard: Ratatui Motor Diagnostics"
            }
        }
        "widget_delay_title" => {
            if is_zh {
                "延时：Ratatui 延时触发器"
            } else {
                "delay: Ratatui Delayed Trigger"
            }
        }
        "widget_dial_title" => {
            if is_zh {
                "仪表盘：Ratatui 转速仪表"
            } else {
                "dial: Ratatui Speed Dial"
            }
        }
        "widget_example_title" => {
            if is_zh {
                "示例：Rust/Ratatui 模块模板"
            } else {
                "example: Rust/Ratatui Module Template"
            }
        }
        "widget_gauge_title" => {
            if is_zh {
                "电量计：Ratatui 电池电量"
            } else {
                "gauge: Ratatui Battery Gauge"
            }
        }
        "widget_image_title" => {
            if is_zh {
                "图像：Ratatui 区域画布"
            } else {
                "image: Ratatui ROI Canvas"
            }
        }
        "widget_joystick_title" => {
            if is_zh {
                "摇杆：Ratatui 画布网格"
            } else {
                "joystick: Ratatui Canvas Grid"
            }
        }
        "widget_knob_title" => {
            if is_zh {
                "旋钮：Ratatui 微调旋钮"
            } else {
                "knob: Ratatui Fine Dial"
            }
        }
        "widget_light_title" => {
            if is_zh {
                "指示灯：Ratatui 状态指示"
            } else {
                "light: Ratatui Status Indicators"
            }
        }
        "widget_pad_title" => {
            if is_zh {
                "触控板：Ratatui 画布输入"
            } else {
                "pad: Ratatui Canvas Input"
            }
        }
        "widget_ring_title" => {
            if is_zh {
                "环形：Ratatui 环形刻度盘"
            } else {
                "ring: Ratatui Ring Dial"
            }
        }
        "widget_slider_title" => {
            if is_zh {
                "滑块：Ratatui PID 参数面板"
            } else {
                "slider: Ratatui PID Parameter Panel"
            }
        }
        "widget_toggle_title" => {
            if is_zh {
                "开关：Ratatui 锁存开关"
            } else {
                "toggle: Ratatui Latched Switch"
            }
        }

        _ => key,
    }
}
