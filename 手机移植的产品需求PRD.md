toolName: view_files
            
status: success
          
            
filePath: d:\workspace\zeroclaw\手机移植技术实现方案.md
          
我来为你编写 Mobile Claw 的 PRD 需求文档。
        
            
toolName: view_files
            
status: success
          
            
filePath: d:\workspace\zeroclaw\AGENTS.md
          
我将为你创建 Mobile Claw 的 PRD 需求文档。

```markdown d:\workspace\zeroclaw\Mobile_Claw_PRD.md
# Mobile Claw 产品需求文档 (PRD)

## 文档信息

| 项目名称 | Mobile Claw - 移动端AI Agent |
|---------|------------------------------|
| 版本 | 1.0.0 |
| 创建日期 | 2026-03-11 |
| 文档状态 | Draft |
| 产品负责人 | ZeroClaw Team |

---

## 1. 产品概述

### 1.1 产品定位

Mobile Claw 是一款运行在移动设备（Android/iOS）上的本地化 AI Agent，作为智能家居控制中心和个人智能助手，通过 AI 能力连接和控制各类智能设备，为用户提供个性化、智能化的生活体验。

### 1.2 核心价值

- **隐私优先**: 所有数据本地存储，主打本地模型推理
- **智能中枢**: 作为移动端网关，连接和控制所有智能设备
- **个性化体验**: AI 学习用户习惯，提供定制化服务
- **跨平台**: 支持 Android 和 iOS 双平台

### 1.3 目标用户

| 用户群体 | 特征描述 | 核心需求 |
|---------|---------|---------|
| 智能家居用户 | 拥有多款智能设备的家庭用户 | 统一控制、智能联动 |
| 隐私敏感用户 | 注重数据隐私的用户 | 本地处理、数据不出设备 |
| 科技爱好者 | 追求新技术体验的用户 | AI能力、自定义扩展 |
| 普通家庭用户 | 对技术不敏感的家庭用户 | 简单易用、语音控制 |

---

## 2. 功能需求

### 2.1 平台支持

#### 2.1.1 双平台适配

| 平台 | 最低版本 | 目标版本 | 特殊要求 |
|-----|---------|---------|---------|
| Android | Android 8.0 (API 26) | Android 14 | 后台服务保活 |
| iOS | iOS 14.0 | iOS 17 | 后台任务支持 |

#### 2.1.2 Android 后台服务

**需求描述**: 提供专门的 Android 后台服务，确保 AI Agent 在后台持续运行。

**功能要点**:
- 前台服务 (Foreground Service) 实现，显示常驻通知
- 服务保活机制，防止系统杀进程
- 低功耗模式，后台运行时降低资源占用
- 开机自启动支持
- 服务重启恢复机制

**技术实现**:
```
Android Service 架构:
├── MobileClawService (Foreground Service)
│   ├── AgentCore (AI 引擎)
│   ├── DeviceManager (设备管理)
│   ├── NetworkGateway (网络网关)
│   └── NotificationManager (通知管理)
├── BootReceiver (开机启动)
├── ServiceMonitor (服务监控)
└── PowerManager (电源管理)
```

#### 2.1.3 iOS 后台 Daemon

**需求描述**: iOS 平台后台运行支持，确保核心功能在后台可用。

**功能要点**:
- Background Tasks 框架支持
- Background Modes 配置
  - Background fetch
  - Background processing
  - Remote notifications
- 低功耗后台运行策略
- 后台任务调度管理

**技术实现**:
```
iOS Background 架构:
├── BackgroundTaskManager
│   ├── ProcessingTask (AI 处理任务)
│   ├── FetchTask (数据同步)
│   └── MaintenanceTask (维护任务)
├── DeviceConnectionManager
│   ├── BluetoothBackground (蓝牙后台)
│   └── NetworkReachability (网络监听)
└── NotificationServiceExtension
```

### 2.2 协议支持

#### 2.2.1 A2A (Agent-to-Agent) 协议

**需求描述**: 支持与其他 AI Agent 的通信协议。

**功能要点**:
- Agent 发现与注册
- Agent 间消息传递
- Agent 能力协商
- 安全认证机制

**接口定义**:
```typescript
interface A2AProtocol {
  agentId: string;
  agentName: string;
  capabilities: string[];
  endpoints: {
    message: string;
    discovery: string;
    negotiate: string;
  };
  authentication: {
    type: 'token' | 'certificate';
    credentials: string;
  };
}

interface A2AMessage {
  from: string;
  to: string;
  type: 'request' | 'response' | 'notification';
  payload: any;
  timestamp: number;
  signature?: string;
}
```

#### 2.2.2 ACP (Agent Communication Protocol)

**需求描述**: 标准化的 Agent 通信协议支持。

**功能要点**:
- 消息格式标准化
- 多种消息类型支持
- 消息路由与转发
- 消息确认与重试

**消息类型**:
| 类型 | 描述 | 使用场景 |
|-----|------|---------|
| COMMAND | 命令消息 | 设备控制指令 |
| QUERY | 查询消息 | 状态查询 |
| EVENT | 事件消息 | 设备状态变化通知 |
| RESPONSE | 响应消息 | 命令/查询响应 |
| ERROR | 错误消息 | 异常情况通知 |

#### 2.2.3 MCP (Model Context Protocol)

**需求描述**: 支持 MCP 协议，实现模型上下文共享。

**功能要点**:
- Context 定义与管理
- Context 序列化与传输
- Context 版本控制
- Context 同步机制

**Context 结构**:
```typescript
interface MCPContext {
  id: string;
  version: string;
  type: 'conversation' | 'device_state' | 'user_preference' | 'environment';
  data: {
    content: any;
    metadata: Record<string, any>;
    timestamp: number;
  };
  permissions: {
    read: string[];
    write: string[];
  };
}
```

### 2.3 模型支持

#### 2.3.1 本地模型（主推）

**需求描述**: 主打本地模型推理，保护用户隐私。

**支持的本地模型**:

| 模型类型 | 模型示例 | 硬件要求 | 使用场景 |
|---------|---------|---------|---------|
| 小型模型 | Qwen-1.8B, Phi-2 | 4GB RAM | 快速响应 |
| 中型模型 | Qwen-7B, Llama-3-8B | 8GB RAM | 日常对话 |
| 量化模型 | Qwen-7B-INT4, Llama-3-8B-Q4 | 6GB RAM | 平衡性能 |

**本地模型功能**:
- 模型下载与管理
- 模型量化支持 (INT4/INT8)
- 模型热切换
- 推理优化 (GPU 加速)
- 离线推理能力

**隐私保护措施**:
- 所有对话数据本地存储
- 用户画像本地生成
- 敏感信息不上传云端
- 数据加密存储

#### 2.3.2 云端模型（备选）

**需求描述**: 支持云端模型作为备选方案。

**支持的云服务商**:
- OpenAI (GPT-4, GPT-3.5)
- Anthropic (Claude)
- 阿里云 (通义千问)
- 百度 (文心一言)
- 自定义 API 端点

**云端模型配置**:
```typescript
interface CloudModelConfig {
  provider: string;
  model: string;
  apiKey?: string;
  baseUrl?: string;
  timeout: number;
  retryCount: number;
  fallbackToLocal: boolean;
}
```

#### 2.3.3 混合推理策略

**需求描述**: 智能选择本地/云端模型。

**策略规则**:
| 场景 | 优先选择 | 原因 |
|-----|---------|------|
| 简单对话 | 本地模型 | 快速响应，隐私保护 |
| 复杂推理 | 云端模型 | 能力更强 |
| 设备控制 | 本地模型 | 低延迟要求 |
| 无网络环境 | 本地模型 | 唯一选择 |
| 敏感信息处理 | 本地模型 | 隐私保护 |

### 2.4 网络网关功能

#### 2.4.1 多网络连接支持

**需求描述**: 作为移动网关，支持多种网络连接方式。

**网络类型**:

| 网络类型 | 协议 | 使用场景 | 优先级 |
|---------|------|---------|-------|
| WiFi | TCP/UDP, HTTP/WS | 家庭设备控制 | 高 |
| 蓝牙 | BLE, Classic | 近距离设备控制 | 高 |
| USB网络 | RNDIS, AOA | 有线连接设备 | 中 |
| 本地网络 | mDNS, SSDP | 设备发现 | 高 |
| 蜂窝网络 | HTTPS | 远程控制 | 低 |

**网络架构**:
```
┌─────────────────────────────────────────────────────────┐
│                    Mobile Claw Gateway                  │
├─────────────────────────────────────────────────────────┤
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐   │
│  │  WiFi   │  │Bluetooth│  │   USB   │  │ Cellular│   │
│  │ Manager │  │ Manager │  │ Manager │  │ Manager │   │
│  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘   │
│       │            │            │            │         │
│       └────────────┴─────┬──────┴────────────┘         │
│                          │                              │
│              ┌───────────▼───────────┐                 │
│              │   Network Router      │                 │
│              │   (智能路由选择)       │                 │
│              └───────────┬───────────┘                 │
│                          │                              │
│              ┌───────────▼───────────┐                 │
│              │   Device Manager      │                 │
│              └───────────────────────┘                 │
└─────────────────────────────────────────────────────────┘
```

#### 2.4.2 设备发现

**需求描述**: 自动发现局域网内的智能设备。

**发现协议**:
- mDNS/Bonjour (Apple 设备)
- SSDP (UPnP 设备)
- BLE Scan (蓝牙设备)
- CoAP Discovery (IoT 设备)
- 自定义发现协议

**设备发现流程**:
```
1. 启动扫描
   ├── WiFi 扫描 (mDNS + SSDP)
   ├── 蓝牙扫描 (BLE)
   └── USB 设备枚举
2. 设备识别
   ├── 解析设备信息
   ├── 设备类型分类
   └── 能力探测
3. 设备注册
   ├── 添加到设备列表
   ├── 建立连接
   └── 状态同步
```

#### 2.4.3 连接管理

**需求描述**: 管理设备连接状态和生命周期。

**连接状态**:
| 状态 | 描述 | 用户可见 |
|-----|------|---------|
| DISCOVERED | 已发现，未连接 | 是 |
| CONNECTING | 连接中 | 是 |
| CONNECTED | 已连接 | 是 |
| DISCONNECTED | 已断开 | 是 |
| ERROR | 连接错误 | 是 |
| UNAVAILABLE | 不可用 | 是 |

**连接管理功能**:
- 自动重连机制
- 连接池管理
- 心跳检测
- 连接质量监控
- 多设备并发连接

### 2.5 设备控制工具集

#### 2.5.1 工具框架

**需求描述**: 提供可扩展的设备控制工具框架。

**工具类型**:
```typescript
interface DeviceTool {
  id: string;
  name: string;
  description: string;
  category: ToolCategory;
  deviceTypes: string[];
  parameters: ToolParameter[];
  execute: (params: any) => Promise<ToolResult>;
}

enum ToolCategory {
  CONTROL = 'control',      // 控制类
  QUERY = 'query',          // 查询类
  CONFIGURATION = 'config', // 配置类
  AUTOMATION = 'automation' // 自动化类
}
```

#### 2.5.2 设备发现工具

**工具列表**:

| 工具名称 | 功能描述 | 参数 |
|---------|---------|------|
| scan_wifi_devices | 扫描 WiFi 设备 | timeout, filter |
| scan_bluetooth_devices | 扫描蓝牙设备 | timeout, rssi_threshold |
| scan_local_network | 扫描本地网络 | ip_range, ports |
| discover_device | 发现特定设备 | device_type, protocol |
| get_device_info | 获取设备信息 | device_id |

#### 2.5.3 连接管理工具

**工具列表**:

| 工具名称 | 功能描述 | 参数 |
|---------|---------|------|
| connect_device | 连接设备 | device_id, config |
| disconnect_device | 断开设备 | device_id |
| pair_device | 配对设备 | device_id, pin_code |
| forget_device | 忘记设备 | device_id |
| get_connection_status | 获取连接状态 | device_id |
| set_preferred_network | 设置首选网络 | device_id, network_type |

#### 2.5.4 设备控制工具

**通用控制工具**:

| 工具名称 | 功能描述 | 适用设备 |
|---------|---------|---------|
| device_on | 开启设备 | 通用 |
| device_off | 关闭设备 | 通用 |
| get_device_status | 获取设备状态 | 通用 |
| set_device_mode | 设置设备模式 | 通用 |

**空调控制工具**:

| 工具名称 | 功能描述 | 参数 |
|---------|---------|------|
| ac_set_temperature | 设置温度 | temperature (16-30) |
| ac_set_mode | 设置模式 | mode (cool/heat/auto/fan/dry) |
| ac_set_fan_speed | 设置风速 | speed (low/medium/high/auto) |
| ac_set_swing | 设置摆风 | enabled (boolean) |
| ac_set_timer | 设置定时 | duration (minutes) |
| ac_get_temperature | 获取当前温度 | - |
| ac_get_mode | 获取当前模式 | - |

**电视控制工具**:

| 工具名称 | 功能描述 | 参数 |
|---------|---------|------|
| tv_power | 电源控制 | action (on/off/toggle) |
| tv_set_volume | 设置音量 | volume (0-100) |
| tv_set_channel | 设置频道 | channel (number/name) |
| tv_set_input | 设置输入源 | input (hdmi1/hdmi2/usb/app) |
| tv_launch_app | 启动应用 | app_name |
| tv_search_content | 搜索内容 | keyword |
| tv_play_pause | 播放/暂停 | - |
| tv_get_current_program | 获取当前节目 | - |

**摄像头控制工具**:

| 工具名称 | 功能描述 | 参数 |
|---------|---------|------|
| camera_start_recording | 开始录制 | device_id, duration? |
| camera_stop_recording | 停止录制 | device_id |
| camera_take_snapshot | 截图 | device_id |
| camera_set_motion_detection | 设置移动侦测 | enabled, sensitivity? |
| camera_set_night_mode | 设置夜视模式 | enabled |
| camera_ptz_control | 云台控制 | direction, speed |
| camera_get_stream_url | 获取直播地址 | device_id |

**灯光控制工具**:

| 工具名称 | 功能描述 | 参数 |
|---------|---------|------|
| light_set_brightness | 设置亮度 | brightness (0-100) |
| light_set_color | 设置颜色 | color (hex/rgb) |
| light_set_color_temp | 设置色温 | temperature (2700-6500K) |
| light_set_scene | 设置场景 | scene_name |
| light_dim | 调光 | direction (up/down), step |

### 2.6 AI Agent 智能能力

#### 2.6.1 用户画像构建

**需求描述**: AI Agent 学习和记忆用户习惯、偏好。

**画像维度**:

| 维度 | 数据类型 | 示例 |
|-----|---------|------|
| 生活习惯 | 时间序列 | 起床时间、睡眠时间 |
| 温度偏好 | 数值范围 | 夏季 24-26°C，冬季 20-22°C |
| 娱乐偏好 | 分类标签 | 喜欢脱口秀、不喜欢恐怖片 |
| 设备使用习惯 | 频率统计 | 常用设备、常用功能 |
| 情绪模式 | 情绪标签 | 压力大时喜欢安静环境 |
| 场景偏好 | 场景配置 | 工作模式、睡眠模式 |

**画像存储结构**:
```typescript
interface UserProfile {
  userId: string;
  basicInfo: {
    name: string;
    timezone: string;
    language: string;
  };
  preferences: {
    temperature: {
      summer: { min: number; max: number; preferred: number };
      winter: { min: number; max: number; preferred: number };
    };
    entertainment: {
      genres: string[];
      dislikes: string[];
      watchTime: string[];
    };
    lighting: {
      morning: { brightness: number; colorTemp: number };
      evening: { brightness: number; colorTemp: number };
    };
  };
  habits: {
    wakeUpTime: string;
    sleepTime: string;
    deviceUsagePatterns: DeviceUsagePattern[];
  };
  emotionalPatterns: {
    stressTriggers: string[];
    relaxationPreferences: string[];
  };
  customScenes: CustomScene[];
}
```

#### 2.6.2 智能推荐系统

**需求描述**: 基于用户画像和环境状态，主动推荐操作。

**推荐场景**:

| 场景 | 触发条件 | 推荐动作 |
|-----|---------|---------|
| 回家模式 | GPS 到家 + 时间判断 | 开灯、开空调、播放音乐 |
| 睡眠模式 | 时间接近睡眠时间 | 关灯、调低音量、设置空调 |
| 离家模式 | GPS 离家 | 关闭所有设备、开启安防 |
| 节能模式 | 电量低/电费高峰 | 调整空调温度、关闭非必要设备 |
| 放松模式 | 检测到压力情绪 | 播放轻音乐、调暗灯光 |

**推荐算法**:
```
推荐流程:
1. 环境感知
   ├── 时间 (当前时间、星期、季节)
   ├── 位置 (GPS、WiFi 定位)
   ├── 天气 (温度、湿度、天气状况)
   ├── 设备状态 (在线设备、当前状态)
   └── 用户状态 (日程、情绪推测)

2. 意图推断
   ├── 历史行为匹配
   ├── 规则引擎匹配
   └── ML 模型预测

3. 推荐生成
   ├── 计算推荐分数
   ├── 排序推荐列表
   └── 过滤不可行项

4. 执行/展示
   ├── 自动执行 (高置信度)
   └── 询问用户 (中置信度)
```

#### 2.6.3 情绪感知与响应

**需求描述**: AI Agent 能够感知用户情绪并做出响应。

**情绪识别方式**:
- 对话内容分析 (文本情感分析)
- 语音语调分析 (语音情感识别)
- 行为模式分析 (使用习惯变化)
- 显式反馈 (用户直接表达)

**情绪响应策略**:

| 情绪状态 | 响应策略 | 具体行动 |
|---------|---------|---------|
| 开心 | 增强体验 | 播放欢快音乐、调亮灯光 |
| 疲劳 | 放松环境 | 调暗灯光、播放轻音乐、建议休息 |
| 压力大 | 减压措施 | 播放脱口秀/小品、调整舒适温度 |
| 悲伤 | 安慰陪伴 | 播放舒缓音乐、温暖灯光、主动关怀 |
| 焦虑 | 平静环境 | 降低噪音、调暗灯光、播放白噪音 |

**示例场景**:
```
用户: "我今天心情不太好，感觉很累。"
AI Agent 响应:
1. 情绪识别: 疲劳 + 轻度低落
2. 推荐策略: 放松环境 + 娱乐调节
3. 执行动作:
   - 调暗客厅灯光到 30%
   - 空调调整到舒适温度 24°C
   - 推荐节目: "为您推荐一些轻松的脱口秀节目"
   - 询问: "要不要帮您打开电视播放《脱口秀大会》？"
```

#### 2.6.4 场景自动化

**需求描述**: 支持用户自定义场景和自动化规则。

**场景配置**:
```typescript
interface Scene {
  id: string;
  name: string;
  description: string;
  icon: string;
  triggers: SceneTrigger[];
  actions: SceneAction[];
  conditions?: SceneCondition[];
  schedule?: {
    enabled: boolean;
    cron: string;
  };
}

interface SceneTrigger {
  type: 'manual' | 'schedule' | 'location' | 'device' | 'voice';
  config: {
    // 根据类型不同配置不同
  };
}

interface SceneAction {
  deviceId: string;
  action: string;
  parameters: Record<string, any>;
  delay?: number; // 延迟执行（秒）
}
```

**预设场景**:

| 场景名称 | 触发方式 | 动作列表 |
|---------|---------|---------|
| 起床模式 | 定时 7:00 | 开灯(渐亮)、开窗帘、播放新闻 |
| 工作模式 | 手动/语音 | 关闭干扰设备、设置勿扰 |
| 电影模式 | 手动/语音 | 关灯、关窗帘、电视开、音响开 |
| 睡眠模式 | 定时 22:30 | 关闭所有灯、空调调睡眠模式 |
| 离家模式 | GPS 离家 | 关闭所有设备、开启摄像头 |

### 2.7 数据存储与隐私

#### 2.7.1 本地数据存储

**需求描述**: 所有数据本地存储，不上传云端。

**存储内容**:

| 数据类型 | 存储位置 | 加密 | 保留策略 |
|---------|---------|------|---------|
| 用户配置 | SQLite | AES-256 | 永久 |
| 对话历史 | SQLite | AES-256 | 可配置 |
| 设备信息 | SQLite | 否 | 永久 |
| 用户画像 | SQLite | AES-256 | 永久 |
| 场景配置 | SQLite | 否 | 永久 |
| 模型缓存 | 文件系统 | 否 | 可清理 |
| 临时数据 | 内存 | - | 会话结束清除 |

**存储架构**:
```
本地存储结构:
├── data/
│   ├── config.db          # 配置数据库
│   ├── history.db         # 历史数据库
│   ├── devices.db         # 设备数据库
│   ├── profile.db         # 用户画像数据库
│   └── scenes.db          # 场景数据库
├── models/
│   ├── local/             # 本地模型
│   └── cache/             # 模型缓存
├── media/
│   ├── recordings/        # 录制文件
│   └── snapshots/         # 截图文件
└── logs/
    ├── app.log            # 应用日志
    └── device.log         # 设备日志
```

#### 2.7.2 隐私保护措施

**需求描述**: 确保用户隐私安全。

**保护措施**:

| 措施 | 描述 | 实现方式 |
|-----|------|---------|
| 数据加密 | 敏感数据加密存储 | AES-256-GCM |
| 本地处理 | 所有 AI 推理本地进行 | 本地模型优先 |
| 数据隔离 | 不同用户数据隔离 | 独立数据库文件 |
| 权限控制 | 最小权限原则 | 运行时权限申请 |
| 数据脱敏 | 日志中脱敏敏感信息 | 正则替换 |
| 安全传输 | 设备通信加密 | TLS 1.3 |
| 数据导出 | 支持用户导出数据 | JSON 格式 |
| 数据删除 | 支持用户删除数据 | 完整删除 + 覆盖 |

### 2.8 用户界面

#### 2.8.1 主要界面

**首页 (Dashboard)**:
- 快捷设备控制卡片
- 场景快捷入口
- AI 对话入口
- 常用功能入口
- 设备状态概览

**设备管理页**:
- 设备列表 (按房间/类型分类)
- 设备详情页
- 设备控制面板
- 设备添加向导

**AI 对话页**:
- 对话界面
- 上下文信息展示
- 工具调用可视化
- 推荐操作展示

**场景管理页**:
- 场景列表
- 场景创建/编辑
- 自动化规则配置
- 场景执行历史

**设置页**:
- 模型配置
- 网络配置
- 隐私设置
- 用户画像查看/编辑
- 数据管理

#### 2.8.2 交互方式

| 交互方式 | 描述 | 使用场景 |
|---------|------|---------|
| 触控操作 | 点击、滑动、长按 | 主要交互方式 |
| 语音控制 | 语音命令识别 | 解放双手场景 |
| 文字对话 | 自然语言对话 | 复杂操作、信息查询 |
| 手势控制 | 快捷手势 | 快速操作 |
| 自动化 | 无需用户操作 | 日常自动化场景 |

---

## 3. 非功能需求

### 3.1 性能要求

| 指标 | 目标值 | 测量方法 |
|-----|-------|---------|
| 应用启动时间 | < 2s | 冷启动测量 |
| 设备发现时间 | < 5s | 扫描完成时间 |
| 设备控制响应 | < 500ms | 指令发出到执行 |
| AI 响应时间 (本地) | < 2s | 首字响应时间 |
| AI 响应时间 (云端) | < 3s | 首字响应时间 |
| 内存占用 (前台) | < 200MB | 系统监控 |
| 内存占用 (后台) | < 50MB | 系统监控 |
| CPU 占用 (后台) | < 5% | 系统监控 |
| 电池消耗 (后台) | < 2%/小时 | 电池监控 |

### 3.2 稳定性要求

| 指标 | 目标值 |
|-----|-------|
| 崩溃率 | < 0.1% |
| ANR 率 (Android) | < 0.05% |
| 后台服务存活率 | > 99% |
| 设备连接成功率 | > 95% |
| 自动恢复成功率 | > 98% |

### 3.3 安全要求

| 要求 | 描述 |
|-----|------|
| 数据加密 | 敏感数据 AES-256 加密存储 |
| 传输加密 | TLS 1.3 加密传输 |
| 认证机制 | 设备认证 + 用户认证 |
| 权限管理 | 最小权限 + 运行时授权 |
| 代码安全 | 代码混淆 + 反调试 |
| 安全审计 | 安全日志 + 异常检测 |

### 3.4 兼容性要求

**设备兼容性**:
- 支持主流智能家居品牌 (小米、华为、涂鸦、HomeKit 等)
- 支持标准协议设备 (Zigbee、Z-Wave、WiFi、蓝牙)
- 支持自定义协议扩展

**系统兼容性**:
- Android 8.0 - Android 14
- iOS 14.0 - iOS 17
- 适配主流屏幕尺寸

---

## 4. 技术架构

### 4.1 整体架构

```
┌─────────────────────────────────────────────────────────────────┐
│                        Mobile Claw App                          │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    Presentation Layer                    │   │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐       │   │
│  │  │  React  │ │  Voice  │ │  Scene  │ │ Device  │       │   │
│  │  │   UI    │ │   UI    │ │   UI    │ │   UI    │       │   │
│  │  └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘       │   │
│  └───────┴──────────┴──────────┴──────────┴───────────────┘   │
│                              │                                  │
│  ┌───────────────────────────▼─────────────────────────────┐   │
│  │                    Application Layer                     │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐   │   │
│  │  │   AI     │ │  Device  │ │  Scene   │ │  User    │   │   │
│  │  │  Agent   │ │ Manager  │ │ Manager  │ │ Profile  │   │   │
│  │  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘   │   │
│  └───────┴────────────┴────────────┴────────────┴───────────┘   │
│                              │                                  │
│  ┌───────────────────────────▼─────────────────────────────┐   │
│  │                     Service Layer                        │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐   │   │
│  │  │ Android  │ │   iOS    │ │  Model   │ │ Network  │   │   │
│  │  │ Service  │ │  Daemon  │ │ Runtime  │ │ Gateway  │   │   │
│  │  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘   │   │
│  └───────┴────────────┴────────────┴────────────┴───────────┘   │
│                              │                                  │
│  ┌───────────────────────────▼─────────────────────────────┐   │
│  │                     Core Layer                           │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐   │   │
│  │  │ Protocol │ │  Tool    │ │  Memory  │ │ Security │   │   │
│  │  │  Stack   │ │ Registry │ │  System  │ │  Module  │   │   │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘   │   │
│  └─────────────────────────────────────────────────────────┘   │
│                              │                                  │
│  ┌───────────────────────────▼─────────────────────────────┐   │
│  │                   Platform Layer                         │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐   │   │
│  │  │  SQLite  │ │  File    │ │  BLE/    │ │ Network  │   │   │
│  │  │ Storage  │ │ Storage  │ │  WiFi    │ │  Stack   │   │   │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘   │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

### 4.2 技术选型

| 层级 | 技术选型 | 说明 |
|-----|---------|------|
| 前端框架 | React Native / Flutter | 跨平台 UI |
| 后端语言 | Rust / Kotlin / Swift | 核心逻辑 |
| 本地数据库 | SQLite | 数据存储 |
| 本地模型 | llama.cpp / MLC LLM | 模型推理 |
| 网络框架 | Tokio / URLSession | 网络通信 |
| 蓝牙框架 | BlueZ / CoreBluetooth | 蓝牙通信 |

---

## 5. 里程碑规划

### 5.1 版本规划

| 版本 | 时间 | 主要功能 |
|-----|------|---------|
| v1.0 | M1-M3 | 基础框架、设备连接、基础控制 |
| v1.5 | M4-M5 | 本地模型、AI 对话、用户画像 |
| v2.0 | M6-M8 | 智能推荐、场景自动化、情绪感知 |
| v2.5 | M9-M10 | A2A/ACP/MCP 协议、多设备协同 |

### 5.2 详细里程碑

**M1 (第1-2周): 项目初始化**
- 项目架构搭建
- 开发环境配置
- 基础框架实现

**M2 (第3-4周): 设备连接**
- WiFi/蓝牙设备发现
- 设备连接管理
- 基础设备控制

**M3 (第5-6周): 后台服务**
- Android Service 实现
- iOS Daemon 实现
- 后台保活机制

**M4 (第7-8周): 本地模型**
- 模型集成
- 推理优化
- 对话功能

**M5 (第9-10周): AI 能力**
- 用户画像构建
- 基础推荐系统
- 记忆系统

**M6 (第11-12周): 智能推荐**
- 场景推荐
- 自动化规则
- 智能控制

**M7 (第13-14周): 情绪感知**
- 情绪识别
- 响应策略
- 个性化服务

**M8 (第15-16周): 优化完善**
- 性能优化
- 用户体验优化
- Bug 修复

**M9 (第17-18周): 协议支持**
- A2A 协议
- ACP 协议
- MCP 协议

**M10 (第19-20周): 发布准备**
- 测试完善
- 文档编写
- 应用商店提交

---

## 6. 风险与应对

| 风险 | 影响 | 可能性 | 应对措施 |
|-----|------|-------|---------|
| 本地模型性能不足 | 高 | 中 | 提供云端备选、模型优化 |
| 后台服务被系统杀掉 | 高 | 中 | 多重保活机制、用户引导 |
| 设备兼容性问题 | 中 | 高 | 协议适配层、持续更新 |
| 隐私合规风险 | 高 | 低 | 隐私设计、合规审查 |
| 电池消耗过大 | 中 | 中 | 功耗优化、后台策略 |

---

## 7. 附录

### 7.1 术语表

| 术语 | 解释 |
|-----|------|
| A2A | Agent-to-Agent，Agent 间通信协议 |
| ACP | Agent Communication Protocol，Agent 通信协议 |
| MCP | Model Context Protocol，模型上下文协议 |
| BLE | Bluetooth Low Energy，低功耗蓝牙 |
| mDNS | Multicast DNS，多播 DNS |
| SSDP | Simple Service Discovery Protocol，简单服务发现协议 |

### 7.2 参考文档

- ZeroClaw 技术实现方案
- Android Service 官方文档
- iOS Background Tasks 官方文档
- MCP 协议规范
- 智能家居协议标准
```

以上是 Mobile Claw 的完整 PRD 需求文档，涵盖了：

1. **平台支持**: Android/iOS 双平台，包含后台服务详细设计
2. **协议支持**: A2A、ACP、MCP 三种协议的接口定义
3. **模型支持**: 本地模型为主、云端为辅的混合策略
4. **网络网关**: WiFi、蓝牙、USB、本地网络等多连接方式
5. **设备工具集**: 设备发现、连接管理、各类设备控制工具
6. **AI 智能能力**: 用户画像、智能推荐、情绪感知、场景自动化
7. **隐私保护**: 本地存储、数据加密等安全措施
8. **技术架构**: 分层架构设计和技术选型
9. **里程碑规划**: 20周的开发计划

如需调整或补充任何内容，请告诉我。