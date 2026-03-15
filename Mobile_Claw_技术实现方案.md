# Mobile Claw 技术实现方案设计文档

## 文档信息

| 项目名称 | Mobile Claw - 移动端AI Agent |
|---------|------------------------------|
| 版本 | 1.0.0 |
| 创建日期 | 2026-03-11 |
| 文档状态 | Draft |
| 技术负责人 | ZeroClaw Team |

---

## 1. 技术架构总览

### 1.1 整体架构设计

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Mobile Claw Application                           │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │                         Presentation Layer                             │ │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐    │ │
│  │  │   Chat UI   │ │  Device UI  │ │  Scene UI   │ │ Settings UI │    │ │
│  │  └──────┬──────┘ └──────┬──────┘ └──────┬──────┘ └──────┬──────┘    │ │
│  │         │               │               │               │            │ │
│  │  ┌──────▼───────────────▼───────────────▼───────────────▼──────┐    │ │
│  │  │              React Native / Flutter Bridge                  │    │ │
│  │  └────────────────────────────┬────────────────────────────────┘    │ │
│  └───────────────────────────────┼─────────────────────────────────────┘ │
│                                  │                                         │
│  ┌───────────────────────────────▼─────────────────────────────────────┐ │
│  │                        Application Layer                             │ │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐    │ │
│  │  │ AI Agent    │ │  Device     │ │   Scene     │ │   User      │    │ │
│  │  │ Manager     │ │  Manager    │ │  Manager    │ │  Profiler   │    │ │
│  │  └──────┬──────┘ └──────┬──────┘ └──────┬──────┘ └──────┬──────┘    │ │
│  │         │               │               │               │            │ │
│  │  ┌──────▼───────────────▼───────────────▼───────────────▼──────┐    │ │
│  │  │                   Business Logic Core                        │    │ │
│  │  └────────────────────────────┬────────────────────────────────┘    │ │
│  └───────────────────────────────┼─────────────────────────────────────┘ │
│                                  │                                         │
│  ┌───────────────────────────────▼─────────────────────────────────────┐ │
│  │                          Service Layer                               │ │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐    │ │
│  │  │   Model     │ │  Protocol   │ │   Tool      │ │   Memory    │    │ │
│  │  │  Runtime    │ │   Stack     │ │  Registry   │ │   System    │    │ │
│  │  └──────┬──────┘ └──────┬──────┘ └──────┬──────┘ └──────┬──────┘    │ │
│  │         │               │               │               │            │ │
│  │  ┌──────▼───────────────▼───────────────▼───────────────▼──────┐    │ │
│  │  │              Background Service (Android/iOS)                │    │ │
│  │  └────────────────────────────┬────────────────────────────────┘    │ │
│  └───────────────────────────────┼─────────────────────────────────────┘ │
│                                  │                                         │
│  ┌───────────────────────────────▼─────────────────────────────────────┐ │
│  │                           Core Layer                                 │ │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐    │ │
│  │  │   Network   │ │   Storage   │ │  Security   │ │    Utils    │    │ │
│  │  │   Stack     │ │   Engine    │ │   Module    │ │   Library   │    │ │
│  │  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘    │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐ │
│  │                        Platform Layer                                │ │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐    │ │
│  │  │   Android   │ │    iOS      │ │   SQLite    │ │  Keychain   │    │ │
│  │  │    SDK      │ │    SDK      │ │   Native    │ │   /Keystore │    │ │
│  │  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘    │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 1.2 技术选型

| 层级 | 技术选型 | 说明 | 选型理由 |
|-----|---------|------|---------|
| **跨平台框架** | Flutter 3.x / React Native | UI 层跨平台 | 成熟稳定，生态丰富 |
| **核心语言** | Dart / TypeScript + Rust | 业务逻辑 | 高性能，类型安全 |
| **本地模型** | llama.cpp / MLC LLM | 本地推理 | 移动端优化，支持量化 |
| **数据库** | SQLite + Drift / Realm | 本地存储 | 轻量级，高性能 |
| **网络框架** | Dio / Alamofire | 网络请求 | 功能完善，易用 |
| **蓝牙框架** | flutter_blue_plus / CoreBluetooth | 蓝牙通信 | 跨平台支持 |
| **状态管理** | Riverpod / Bloc | 应用状态 | 响应式，可测试 |
| **安全存储** | flutter_secure_storage | 敏感数据 | 系统级加密 |

### 1.3 模块依赖关系

```
┌─────────────────────────────────────────────────────────────────┐
│                        Module Dependencies                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   ┌──────────┐     ┌──────────┐     ┌──────────┐              │
│   │   UI     │────▶│  Agent   │────▶│  Model   │              │
│   │ Layer    │     │ Manager  │     │ Runtime  │              │
│   └──────────┘     └──────────┘     └──────────┘              │
│        │                │                │                     │
│        │                ▼                │                     │
│        │          ┌──────────┐          │                     │
│        │          │  Device  │          │                     │
│        │          │ Manager  │          │                     │
│        │          └──────────┘          │                     │
│        │                │                │                     │
│        ▼                ▼                ▼                     │
│   ┌──────────┐     ┌──────────┐     ┌──────────┐              │
│   │  Scene   │     │   Tool   │     │  Memory  │              │
│   │ Manager  │     │ Registry │     │  System  │              │
│   └──────────┘     └──────────┘     └──────────┘              │
│        │                │                │                     │
│        └────────────────┼────────────────┘                     │
│                         ▼                                      │
│                   ┌──────────┐                                │
│                   │ Protocol │                                │
│                   │  Stack   │                                │
│                   └──────────┘                                │
│                         │                                      │
│                         ▼                                      │
│                   ┌──────────┐                                │
│                   │  Core    │                                │
│                   │ Services │                                │
│                   └──────────┘                                │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 2. 核心模块详细设计

### 2.1 AI Agent Manager

#### 2.1.1 模块职责

- 管理 AI Agent 生命周期
- 协调模型推理和工具调用
- 处理用户对话和上下文
- 实现混合推理策略

#### 2.1.2 类图设计

```
┌─────────────────────────────────────────────────────────────────┐
│                      AI Agent Manager                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                  <<interface>> IAgentManager             │   │
│  ├─────────────────────────────────────────────────────────┤   │
│  │ + initialize(config: AgentConfig): Future<void>         │   │
│  │ + sendMessage(message: String): Future<AgentResponse>   │  
│  │ + streamMessage(message: String): Stream<String>        │   │
│  │ + executeTool(toolName: String, params: Map): Future    │   │
│  │ + getConversationHistory(): List<Message>               │   │
│  │ + clearHistory(): void                                  │   │
│  │ + updateConfig(config: AgentConfig): void               │   │
│  └─────────────────────────────────────────────────────────┘   │
│                              △                                  │
│                              │                                  │
│  ┌───────────────────────────┴───────────────────────────┐     │
│  │                 AgentManagerImpl                       │     │
│  ├─────────────────────────────────────────────────────────┤   │
│  │ - modelRuntime: ModelRuntime                           │   │
│  │ - toolRegistry: ToolRegistry                           │   │
│  │ - memorySystem: MemorySystem                           │   │
│  │ - conversationHistory: List<Message>                   │   │
│  │ - config: AgentConfig                                  │   │
│  │ - state: AgentState                                    │   │
│  ├─────────────────────────────────────────────────────────┤   │
│  │ + initialize(config: AgentConfig): Future<void>        │   │
│  │ + sendMessage(message: String): Future<AgentResponse>  │   │
│  │ + streamMessage(message: String): Stream<String>       │   │
│  │ - selectModel(context: MessageContext): ModelType      │   │
│  │ - buildPrompt(message: String): String                 │   │
│  │ - parseToolCalls(response: String): List<ToolCall>     │   │
│  │ - executeToolCalls(calls: List<ToolCall>): List<Result>│   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

#### 2.1.3 核心代码实现

```dart
class AgentManagerImpl implements IAgentManager {
  final ModelRuntime _modelRuntime;
  final ToolRegistry _toolRegistry;
  final MemorySystem _memorySystem;
  final UserProfileManager _profileManager;
  
  List<Message> _conversationHistory = [];
  AgentConfig _config;
  AgentState _state = AgentState.idle;
  
  AgentManagerImpl({
    required ModelRuntime modelRuntime,
    required ToolRegistry toolRegistry,
    required MemorySystem memorySystem,
    required UserProfileManager profileManager,
    required AgentConfig config,
  })  : _modelRuntime = modelRuntime,
        _toolRegistry = toolRegistry,
        _memorySystem = memorySystem,
        _profileManager = profileManager,
        _config = config;

  @override
  Future<void> initialize(AgentConfig config) async {
    _config = config;
    _state = AgentState.initializing;
    
    await _modelRuntime.initialize(config.modelConfig);
    await _loadConversationHistory();
    
    _state = AgentState.ready;
  }

  @override
  Future<AgentResponse> sendMessage(String message) async {
    _state = AgentState.processing;
    
    final userMessage = Message(
      id: _generateId(),
      role: Role.user,
      content: message,
      timestamp: DateTime.now(),
    );
    _conversationHistory.add(userMessage);
    
    final context = await _buildContext(message);
    final modelType = _selectModel(context);
    
    String response;
    List<ToolCall>? toolCalls;
    
    if (modelType == ModelType.local) {
      final result = await _modelRuntime.runLocal(
        prompt: _buildPrompt(message, context),
        config: _config.localModelConfig,
      );
      response = result.content;
      toolCalls = result.toolCalls;
    } else {
      final result = await _modelRuntime.runCloud(
        messages: _conversationHistory,
        config: _config.cloudModelConfig,
      );
      response = result.content;
      toolCalls = result.toolCalls;
    }
    
    List<ToolResult>? toolResults;
    if (toolCalls != null && toolCalls.isNotEmpty) {
      toolResults = await _executeToolCalls(toolCalls);
      if (toolResults.any((r) => r.requiresFollowUp)) {
        response = await _handleToolFollowUp(toolResults);
      }
    }
    
    final assistantMessage = Message(
      id: _generateId(),
      role: Role.assistant,
      content: response,
      timestamp: DateTime.now(),
      toolCalls: toolCalls,
      toolResults: toolResults,
    );
    _conversationHistory.add(assistantMessage);
    
    await _saveConversationHistory();
    await _updateUserProfile(message, response);
    
    _state = AgentState.ready;
    
    return AgentResponse(
      message: assistantMessage,
      toolResults: toolResults,
    );
  }

  @override
  Stream<String> streamMessage(String message) async* {
    _state = AgentState.processing;
    
    final userMessage = Message(
      id: _generateId(),
      role: Role.user,
      content: message,
      timestamp: DateTime.now(),
    );
    _conversationHistory.add(userMessage);
    
    final context = await _buildContext(message);
    final modelType = _selectModel(context);
    
    String fullResponse = '';
    await for (final chunk in _modelRuntime.streamLocal(
      prompt: _buildPrompt(message, context),
      config: _config.localModelConfig,
    )) {
      fullResponse += chunk;
      yield chunk;
    }
    
    final assistantMessage = Message(
      id: _generateId(),
      role: Role.assistant,
      content: fullResponse,
      timestamp: DateTime.now(),
    );
    _conversationHistory.add(assistantMessage);
    
    await _saveConversationHistory();
    _state = AgentState.ready;
  }

  MessageContext _buildContext(String message) async {
    final profile = await _profileManager.getProfile();
    final relevantMemories = await _memorySystem.search(message, limit: 5);
    final deviceStates = await _getRelevantDeviceStates();
    final environment = await _getEnvironmentInfo();
    
    return MessageContext(
      userProfile: profile,
      memories: relevantMemories,
      deviceStates: deviceStates,
      environment: environment,
      conversationHistory: _conversationHistory,
    );
  }

  ModelType _selectModel(MessageContext context) {
    if (!_config.enableCloudModel) {
      return ModelType.local;
    }
    
    if (context.containsSensitiveInfo) {
      return ModelType.local;
    }
    
    if (context.requiresComplexReasoning && _config.cloudModelAvailable) {
      return ModelType.cloud;
    }
    
    if (context.isDeviceControl) {
      return ModelType.local;
    }
    
    return ModelType.local;
  }

  Future<List<ToolResult>> _executeToolCalls(List<ToolCall> calls) async {
    final results = <ToolResult>[];
    
    for (final call in calls) {
      final tool = _toolRegistry.getTool(call.name);
      if (tool == null) {
        results.add(ToolResult(
          toolCallId: call.id,
          success: false,
          error: 'Tool not found: ${call.name}',
        ));
        continue;
      }
      
      try {
        final result = await tool.execute(call.arguments);
        results.add(ToolResult(
          toolCallId: call.id,
          success: true,
          output: result,
        ));
      } catch (e) {
        results.add(ToolResult(
          toolCallId: call.id,
          success: false,
          error: e.toString(),
        ));
      }
    }
    
    return results;
  }
}
```

### 2.2 Device Manager

#### 2.2.1 模块职责

- 设备发现与注册
- 连接生命周期管理
- 设备状态同步
- 多协议适配

#### 2.2.2 架构设计

```
┌─────────────────────────────────────────────────────────────────┐
│                        Device Manager                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                 DeviceManager (Facade)                   │   │
│  ├─────────────────────────────────────────────────────────┤   │
│  │ + discoverDevices(): Future<List<Device>>               │   │
│  │ + connectDevice(deviceId: String): Future<void>         │   │
│  │ + disconnectDevice(deviceId: String): Future<void>      │   │
│  │ + getDevice(deviceId: String): Device?                  │   │
│  │ + getConnectedDevices(): List<Device>                   │   │
│  │ + executeCommand(deviceId, command): Future<Result>     │   │
│  │ + subscribeState(deviceId): Stream<DeviceState>         │   │
│  └─────────────────────────────────────────────────────────┘   │
│                              │                                  │
│         ┌────────────────────┼────────────────────┐            │
│         ▼                    ▼                    ▼            │
│  ┌─────────────┐      ┌─────────────┐      ┌─────────────┐    │
│  │  Discovery  │      │ Connection  │      │   State     │    │
│  │   Engine    │      │   Manager   │      │  Synchronizer│   │
│  └──────┬──────┘      └──────┬──────┘      └──────┬──────┘    │
│         │                    │                    │            │
│         └────────────────────┼────────────────────┘            │
│                              ▼                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                  Protocol Adapters                       │   │
│  ├──────────┬──────────┬──────────┬──────────┬────────────┤   │
│  │  WiFi    │   BLE    │   USB    │  mDNS    │   CoAP     │   │
│  │ Adapter  │ Adapter  │ Adapter  │ Adapter  │  Adapter   │   │
│  └──────────┴──────────┴──────────┴──────────┴────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

#### 2.2.3 核心代码实现

```dart
class DeviceManager {
  final DiscoveryEngine _discoveryEngine;
  final ConnectionManager _connectionManager;
  final StateSynchronizer _stateSynchronizer;
  final DeviceRepository _repository;
  
  final Map<String, Device> _devices = {};
  final Map<String, StreamController<DeviceState>> _stateControllers = {};
  
  DeviceManager({
    required DiscoveryEngine discoveryEngine,
    required ConnectionManager connectionManager,
    required StateSynchronizer stateSynchronizer,
    required DeviceRepository repository,
  })  : _discoveryEngine = discoveryEngine,
        _connectionManager = connectionManager,
        _stateSynchronizer = stateSynchronizer,
        _repository = repository;

  Future<void> initialize() async {
    await _loadCachedDevices();
    await _discoveryEngine.initialize();
    await _connectionManager.initialize();
    
    _discoveryEngine.onDeviceDiscovered.listen(_handleDeviceDiscovered);
    _connectionManager.onConnectionLost.listen(_handleConnectionLost);
    
    await _restoreConnections();
  }

  Future<List<Device>> discoverDevices({
    Duration timeout = const Duration(seconds: 10),
    List<DeviceType>? filter,
  }) async {
    final devices = await _discoveryEngine.scan(timeout: timeout);
    
    for (final device in devices) {
      if (filter == null || filter.contains(device.type)) {
        _devices[device.id] = device;
        await _repository.saveDevice(device);
      }
    }
    
    return devices.where((d) => filter == null || filter!.contains(d.type)).toList();
  }

  Future<void> connectDevice(String deviceId) async {
    final device = _devices[deviceId];
    if (device == null) {
      throw DeviceNotFoundException(deviceId);
    }
    
    if (device.connectionState == ConnectionState.connected) {
      return;
    }
    
    device.connectionState = ConnectionState.connecting;
    _notifyStateChange(device);
    
    try {
      final adapter = _getAdapterForDevice(device);
      await adapter.connect(device);
      
      device.connectionState = ConnectionState.connected;
      device.lastConnected = DateTime.now();
      
      await _repository.updateDevice(device);
      _startStateSync(device);
      
      _notifyStateChange(device);
    } catch (e) {
      device.connectionState = ConnectionState.error;
      device.lastError = e.toString();
      _notifyStateChange(device);
      rethrow;
    }
  }

  Future<void> disconnectDevice(String deviceId) async {
    final device = _devices[deviceId];
    if (device == null) return;
    
    _stopStateSync(device);
    
    final adapter = _getAdapterForDevice(device);
    await adapter.disconnect(device);
    
    device.connectionState = ConnectionState.disconnected;
    await _repository.updateDevice(device);
    _notifyStateChange(device);
  }

  Future<CommandResult> executeCommand(
    String deviceId,
    DeviceCommand command,
  ) async {
    final device = _devices[deviceId];
    if (device == null) {
      throw DeviceNotFoundException(deviceId);
    }
    
    if (device.connectionState != ConnectionState.connected) {
      throw DeviceNotConnectedException(deviceId);
    }
    
    final adapter = _getAdapterForDevice(device);
    
    try {
      final result = await adapter.executeCommand(device, command);
      
      device.lastActivity = DateTime.now();
      await _repository.updateDevice(device);
      
      return result;
    } catch (e) {
      device.lastError = e.toString();
      _notifyStateChange(device);
      rethrow;
    }
  }

  Stream<DeviceState> subscribeState(String deviceId) {
    return _stateControllers.putIfAbsent(
      deviceId,
      () => StreamController<DeviceState>.broadcast(),
    ).stream;
  }

  void _startStateSync(Device device) {
    _stateSynchronizer.startSync(
      device,
      onStateUpdate: (state) {
        device.currentState = state;
        _notifyStateChange(device);
      },
      onError: (error) {
        device.lastError = error.toString();
        _notifyStateChange(device);
      },
    );
  }

  void _notifyStateChange(Device device) {
    final controller = _stateControllers[device.id];
    if (controller != null && !controller.isClosed) {
      controller.add(device.currentState);
    }
  }

  ProtocolAdapter _getAdapterForDevice(Device device) {
    switch (device.protocol) {
      case Protocol.wifi:
        return WifiAdapter();
      case Protocol.bluetooth:
        return BleAdapter();
      case Protocol.usb:
        return UsbAdapter();
      default:
        throw UnsupportedProtocolException(device.protocol);
    }
  }
}
```

### 2.3 Protocol Stack

#### 2.3.1 A2A 协议实现

```dart
class A2AProtocolHandler {
  final String agentId;
  final String agentName;
  final List<String> capabilities;
  final SecurityManager _securityManager;
  final MessageRouter _router;
  
  final Map<String, AgentInfo> _discoveredAgents = {};
  final StreamController<A2AMessage> _messageStream = 
      StreamController<A2AMessage>.broadcast();

  A2AProtocolHandler({
    required this.agentId,
    required this.agentName,
    required this.capabilities,
    required SecurityManager securityManager,
    required MessageRouter router,
  })  : _securityManager = securityManager,
        _router = router;

  Future<void> startDiscovery() async {
    final announcement = A2AAnnouncement(
      agentId: agentId,
      agentName: agentName,
      capabilities: capabilities,
      endpoints: _getEndpoints(),
      timestamp: DateTime.now().millisecondsSinceEpoch,
    );
    
    await _router.broadcast(announcement);
    
    _router.listen<A2AAnnouncement>((message) {
      if (message.agentId != agentId) {
        _handleAgentAnnouncement(message);
      }
    });
  }

  Future<void> sendMessage({
    required String toAgentId,
    required A2AMessageType type,
    required Map<String, dynamic> payload,
  }) async {
    final message = A2AMessage(
      id: _generateMessageId(),
      from: agentId,
      to: toAgentId,
      type: type,
      payload: payload,
      timestamp: DateTime.now().millisecondsSinceEpoch,
    );
    
    final signature = await _securityManager.sign(message.toJson());
    message.signature = signature;
    
    await _router.send(toAgentId, message);
  }

  Stream<A2AMessage> get messageStream => _messageStream.stream;

  Future<A2ANegotiationResult> negotiateCapabilities(
    String targetAgentId,
    List<String> requiredCapabilities,
  ) async {
    final response = await sendMessageAndWait(
      toAgentId: targetAgentId,
      type: A2AMessageType.negotiateRequest,
      payload: {'requiredCapabilities': requiredCapabilities},
      timeout: Duration(seconds: 10),
    );
    
    return A2ANegotiationResult.fromJson(response.payload);
  }

  void _handleAgentAnnouncement(A2AAnnouncement announcement) {
    _discoveredAgents[announcement.agentId] = AgentInfo(
      id: announcement.agentId,
      name: announcement.agentName,
      capabilities: announcement.capabilities,
      endpoints: announcement.endpoints,
      lastSeen: DateTime.now(),
    );
  }
}
```

#### 2.3.2 ACP 协议实现

```dart
class ACPProtocolHandler {
  final MessageSerializer _serializer;
  final MessageValidator _validator;
  final MessageRouter _router;
  final AcknowledgmentManager _ackManager;
  
  final Map<ACPMessageType, MessageHandler> _handlers = {};

  ACPProtocolHandler({
    required MessageSerializer serializer,
    required MessageValidator validator,
    required MessageRouter router,
    required AcknowledgmentManager ackManager,
  })  : _serializer = serializer,
        _validator = validator,
        _router = router,
        _ackManager = ackManager;

  void registerHandler(ACPMessageType type, MessageHandler handler) {
    _handlers[type] = handler;
  }

  Future<ACPResponse> sendCommand(
    String targetId,
    String command,
    Map<String, dynamic> params, {
    Duration timeout = const Duration(seconds: 30),
    bool requireAck = true,
  }) async {
    final message = ACPMessage(
      id: _generateId(),
      type: ACPMessageType.command,
      source: _router.localId,
      target: targetId,
      payload: {
        'command': command,
        'params': params,
      },
      timestamp: DateTime.now().millisecondsSinceEpoch,
      requireAck: requireAck,
    );
    
    final serialized = _serializer.serialize(message);
    await _router.sendRaw(targetId, serialized);
    
    if (requireAck) {
      await _ackManager.waitForAck(message.id, timeout);
    }
    
    return await _waitForResponse(message.id, timeout);
  }

  Future<void> sendEvent(
    String targetId,
    String eventType,
    Map<String, dynamic> data,
  ) async {
    final message = ACPMessage(
      id: _generateId(),
      type: ACPMessageType.event,
      source: _router.localId,
      target: targetId,
      payload: {
        'eventType': eventType,
        'data': data,
      },
      timestamp: DateTime.now().millisecondsSinceEpoch,
      requireAck: false,
    );
    
    await _router.sendRaw(targetId, _serializer.serialize(message));
  }

  void handleMessage(String rawMessage) async {
    final message = _serializer.deserialize(rawMessage);
    
    if (!_validator.validate(message)) {
      await _sendError(message, 'Validation failed');
      return;
    }
    
    if (message.requireAck) {
      await _sendAck(message);
    }
    
    final handler = _handlers[message.type];
    if (handler != null) {
      try {
        final response = await handler.handle(message);
        await _sendResponse(message, response);
      } catch (e) {
        await _sendError(message, e.toString());
      }
    }
  }
}
```

#### 2.3.3 MCP 协议实现

```dart
class MCPProtocolHandler {
  final ContextManager _contextManager;
  final ContextSerializer _serializer;
  final PermissionManager _permissionManager;
  
  final Map<String, MCPContext> _contextCache = {};

  MCPProtocolHandler({
    required ContextManager contextManager,
    required ContextSerializer serializer,
    required PermissionManager permissionManager,
  })  : _contextManager = contextManager,
        _serializer = serializer,
        _permissionManager = permissionManager;

  Future<MCPContext> createContext({
    required String type,
    required Map<String, dynamic> content,
    required Map<String, List<String>> permissions,
  }) async {
    final context = MCPContext(
      id: _generateContextId(),
      version: '1.0',
      type: type,
      data: ContextData(
        content: content,
        metadata: {
          'createdAt': DateTime.now().toIso8601String(),
          'source': 'mobile_claw',
        },
        timestamp: DateTime.now().millisecondsSinceEpoch,
      ),
      permissions: permissions,
    );
    
    await _contextManager.save(context);
    _contextCache[context.id] = context;
    
    return context;
  }

  Future<MCPContext?> getContext(String contextId, String requesterId) async {
    var context = _contextCache[contextId];
    
    if (context == null) {
      context = await _contextManager.load(contextId);
      if (context != null) {
        _contextCache[contextId] = context;
      }
    }
    
    if (context == null) return null;
    
    if (!_permissionManager.hasReadPermission(context, requesterId)) {
      throw PermissionDeniedException('No read permission for context: $contextId');
    }
    
    return context;
  }

  Future<void> updateContext(
    String contextId,
    Map<String, dynamic> newContent,
    String updaterId,
  ) async {
    final context = await getContext(contextId, updaterId);
    if (context == null) {
      throw ContextNotFoundException(contextId);
    }
    
    if (!_permissionManager.hasWritePermission(context, updaterId)) {
      throw PermissionDeniedException('No write permission for context: $contextId');
    }
    
    context.data.content = newContent;
    context.data.metadata['updatedAt'] = DateTime.now().toIso8601String();
    context.data.metadata['updatedBy'] = updaterId;
    context.data.timestamp = DateTime.now().millisecondsSinceEpoch;
    
    final versionParts = context.version.split('.');
    versionParts[1] = (int.parse(versionParts[1]) + 1).toString();
    context.version = versionParts.join('.');
    
    await _contextManager.save(context);
    
    await _notifySubscribers(context);
  }

  Future<void> syncContext(String contextId, String targetAgentId) async {
    final context = _contextCache[contextId] ?? 
        await _contextManager.load(contextId);
    
    if (context == null) {
      throw ContextNotFoundException(contextId);
    }
    
    final serialized = _serializer.serialize(context);
    await _sendToAgent(targetAgentId, serialized);
  }
}
```

### 2.4 Model Runtime

#### 2.4.1 架构设计

```
┌─────────────────────────────────────────────────────────────────┐
│                        Model Runtime                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                  ModelRuntimeManager                     │   │
│  ├─────────────────────────────────────────────────────────┤   │
│  │ + initialize(config: ModelConfig): Future<void>         │   │
│  │ + runLocal(prompt: String): Future<InferenceResult>     │   │
│  │ + streamLocal(prompt: String): Stream<String>           │   │
│  │ + runCloud(messages: List): Future<InferenceResult>     │   │
│  │ + downloadModel(modelId: String): Future<void>          │   │
│  │ + switchModel(modelId: String): Future<void>            │   │
│  │ + getModelStatus(): ModelStatus                         │   │
│  └─────────────────────────────────────────────────────────┘   │
│                              │                                  │
│         ┌────────────────────┴────────────────────┐            │
│         ▼                                         ▼            │
│  ┌─────────────────────┐              ┌─────────────────────┐ │
│  │   LocalModelEngine  │              │  CloudModelEngine   │ │
│  ├─────────────────────┤              ├─────────────────────┤ │
│  │ - llama.cpp FFI     │              │ - OpenAI Client     │ │
│  │ - Model Loader      │              │ - Anthropic Client  │ │
│  │ - Tokenizer         │              │ - Custom API Client │ │
│  │ - GPU Acceleration  │              │ - Retry Logic       │ │
│  │ - Quantization      │              │ - Rate Limiting     │ │
│  └─────────────────────┘              └─────────────────────┘ │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    Model Cache                           │   │
│  ├─────────────────────────────────────────────────────────┤ │
│  │ - KV Cache Management                                   │   │
│  │ - Prompt Cache                                          │   │
│  │ - Response Cache                                        │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

#### 2.4.2 本地模型引擎实现

```dart
class LocalModelEngine {
  final LlamaCppBridge _llamaBridge;
  final ModelDownloader _downloader;
  final ModelCache _cache;
  
  LocalModelConfig? _currentConfig;
  bool _isLoaded = false;

  LocalModelEngine({
    required LlamaCppBridge llamaBridge,
    required ModelDownloader downloader,
    required ModelCache cache,
  })  : _llamaBridge = llamaBridge,
        _downloader = downloader,
        _cache = cache;

  Future<void> initialize(LocalModelConfig config) async {
    _currentConfig = config;
    
    final modelPath = await _ensureModelAvailable(config.modelId);
    
    await _llamaBridge.loadModel(
      modelPath: modelPath,
      contextSize: config.contextSize,
      gpuLayers: config.gpuLayers,
    );
    
    _isLoaded = true;
  }

  Future<String> _ensureModelAvailable(String modelId) async {
    final localPath = await _getLocalModelPath(modelId);
    
    if (await File(localPath).exists()) {
      return localPath;
    }
    
    return await _downloader.download(
      modelId: modelId,
      destination: localPath,
      onProgress: (progress) {
        // Notify download progress
      },
    );
  }

  Future<InferenceResult> runInference({
    required String prompt,
    required InferenceConfig config,
  }) async {
    if (!_isLoaded) {
      throw ModelNotLoadedException();
    }
    
    final cachedResponse = await _cache.getResponse(prompt);
    if (cachedResponse != null && config.useCache) {
      return InferenceResult(
        content: cachedResponse,
        fromCache: true,
        tokensGenerated: 0,
        inferenceTime: Duration.zero,
      );
    }
    
    final stopwatch = Stopwatch()..start();
    
    final result = await _llamaBridge.inference(
      prompt: prompt,
      maxTokens: config.maxTokens,
      temperature: config.temperature,
      topP: config.topP,
      topK: config.topK,
      repeatPenalty: config.repeatPenalty,
      stopSequences: config.stopSequences,
    );
    
    stopwatch.stop();
    
    if (config.useCache) {
      await _cache.setResponse(prompt, result.content);
    }
    
    return InferenceResult(
      content: result.content,
      fromCache: false,
      tokensGenerated: result.tokensGenerated,
      inferenceTime: stopwatch.elapsed,
      toolCalls: _parseToolCalls(result.content),
    );
  }

  Stream<String> streamInference({
    required String prompt,
    required InferenceConfig config,
  }) async* {
    if (!_isLoaded) {
      throw ModelNotLoadedException();
    }
    
    await for (final chunk in _llamaBridge.streamInference(
      prompt: prompt,
      maxTokens: config.maxTokens,
      temperature: config.temperature,
      topP: config.topP,
    )) {
      yield chunk;
    }
  }

  Future<void> switchModel(String modelId) async {
    await _llamaBridge.unloadModel();
    _isLoaded = false;
    
    final newConfig = _currentConfig!.copyWith(modelId: modelId);
    await initialize(newConfig);
  }

  ModelStatus getStatus() {
    return ModelStatus(
      isLoaded: _isLoaded,
      modelId: _currentConfig?.modelId,
      memoryUsage: _isLoaded ? _llamaBridge.getMemoryUsage() : 0,
      gpuUsage: _isLoaded ? _llamaBridge.getGpuUsage() : 0,
    );
  }

  List<ToolCall>? _parseToolCalls(String content) {
    // Parse tool calls from model output
    // Format: <tool_call name="tool_name">{"arg": "value"}</tool_call
    final toolCallPattern = RegExp(
      r'<tool_call name="([^"]+)">(.+?)</tool_call
    );
    
    final matches = toolCallPattern.allMatches(content);
    if (matches.isEmpty) return null;
    
    return matches.map((match) {
      return ToolCall(
        id: _generateId(),
        name: match.group(1)!,
        arguments: jsonDecode(match.group(2)!),
      );
    }).toList();
  }
}
```

### 2.5 Tool Registry

#### 2.5.1 架构设计

```
┌─────────────────────────────────────────────────────────────────┐
│                        Tool Registry                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    ToolRegistry                          │   │
│  ├─────────────────────────────────────────────────────────┤   │
│  │ + registerTool(tool: Tool): void                        │   │
│  │ + unregisterTool(name: String): void                    │   │
│  │ + getTool(name: String): Tool?                          │   │
│  │ + getToolsByCategory(category: Category): List<Tool>    │   │
│  │ + getToolsForDevice(deviceType: String): List<Tool>     │   │
│  │ + executeTool(name: String, params: Map): Future        │   │
│  │ + getToolSchemas(): List<ToolSchema>                    │   │
│  └─────────────────────────────────────────────────────────┘   │
│                              │                                  │
│                              ▼                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                   Tool Categories                        │   │
│  ├──────────────┬──────────────┬──────────────┬────────────┤   │
│  │   Device     │   Query      │   Config     │ Automation │   │
│  │   Control    │   Tools      │   Tools      │   Tools    │   │
│  ├──────────────┼──────────────┼──────────────┼────────────┤   │
│  │ - device_on  │ - get_status │ - set_config │ - create   │   │
│  │ - device_off │ - get_info   │ - update     │   scene    │   │
│  │ - ac_set_temp│ - query_state│ - reset      │ - schedule │   │
│  │ - tv_power   │ - search     │              │ - trigger  │   │
│  │ - light_set  │              │              │            │   │
│  │ - camera_... │              │              │            │   │
│  └──────────────┴──────────────┴──────────────┴────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

#### 2.5.2 工具基类与实现

```dart
abstract class Tool {
  String get name;
  String get description;
  ToolCategory get category;
  List<String> get supportedDeviceTypes;
  Map<String, dynamic> get parameterSchema;
  
  Future<ToolResult> execute(Map<String, dynamic> params);
  bool isAvailable();
  void validateParams(Map<String, dynamic> params);
}

class AcSetTemperatureTool extends Tool {
  final DeviceManager _deviceManager;
  
  AcSetTemperatureTool(this._deviceManager);
  
  @override
  String get name => 'ac_set_temperature';
  
  @override
  String get description => 'Set the temperature of an air conditioner';
  
  @override
  ToolCategory get category => ToolCategory.control;
  
  @override
  List<String> get supportedDeviceTypes => ['air_conditioner', 'ac'];
  
  @override
  Map<String, dynamic> get parameterSchema => {
    'type': 'object',
    'properties': {
      'device_id': {
        'type': 'string',
        'description': 'The ID of the air conditioner device',
      },
      'temperature': {
        'type': 'integer',
        'minimum': 16,
        'maximum': 30,
        'description': 'Target temperature in Celsius (16-30)',
      },
    },
    'required': ['device_id', 'temperature'],
  };
  
  @override
  Future<ToolResult> execute(Map<String, dynamic> params) async {
    validateParams(params);
    
    final deviceId = params['device_id'] as String;
    final temperature = params['temperature'] as int;
    
    try {
      await _deviceManager.executeCommand(
        deviceId,
        DeviceCommand(
          type: 'set_temperature',
          params: {'temperature': temperature},
        ),
      );
      
      return ToolResult(
        success: true,
        output: {
          'device_id': deviceId,
          'temperature': temperature,
          'message': 'Temperature set to $temperature°C',
        },
      );
    } catch (e) {
      return ToolResult(
        success: false,
        error: e.toString(),
      );
    }
  }
  
  @override
  bool isAvailable() {
    return true;
  }
  
  @override
  void validateParams(Map<String, dynamic> params) {
    if (!params.containsKey('device_id')) {
      throw ToolValidationException('device_id is required');
    }
    if (!params.containsKey('temperature')) {
      throw ToolValidationException('temperature is required');
    }
    
    final temp = params['temperature'];
    if (temp is! int || temp < 16 || temp > 30) {
      throw ToolValidationException('temperature must be between 16 and 30');
    }
  }
}

class ToolRegistry {
  final Map<String, Tool> _tools = {};
  final DeviceManager _deviceManager;
  
  ToolRegistry(this._deviceManager) {
    _registerDefaultTools();
  }
  
  void _registerDefaultTools() {
    registerTool(AcSetTemperatureTool(_deviceManager));
    registerTool(AcSetModeTool(_deviceManager));
    registerTool(TvPowerTool(_deviceManager));
    registerTool(TvSetVolumeTool(_deviceManager));
    registerTool(LightSetBrightnessTool(_deviceManager));
    registerTool(CameraStartRecordingTool(_deviceManager));
    registerTool(ScanDevicesTool(_deviceManager));
    registerTool(ConnectDeviceTool(_deviceManager));
    registerTool(GetDeviceStatusTool(_deviceManager));
    registerTool(CreateSceneTool(_deviceManager));
  }
  
  void registerTool(Tool tool) {
    _tools[tool.name] = tool;
  }
  
  void unregisterTool(String name) {
    _tools.remove(name);
  }
  
  Tool? getTool(String name) => _tools[name];
  
  List<Tool> getToolsByCategory(ToolCategory category) {
    return _tools.values.where((t) => t.category == category).toList();
  }
  
  List<Tool> getToolsForDevice(String deviceType) {
    return _tools.values
        .where((t) => t.supportedDeviceTypes.contains(deviceType))
        .toList();
  }
  
  List<ToolSchema> getToolSchemas() {
    return _tools.values.map((tool) {
      return ToolSchema(
        name: tool.name,
        description: tool.description,
        parameters: tool.parameterSchema,
      );
    }).toList();
  }
  
  Future<ToolResult> executeTool(String name, Map<String, dynamic> params) async {
    final tool = getTool(name);
    if (tool == null) {
      throw ToolNotFoundException(name);
    }
    
    if (!tool.isAvailable()) {
      throw ToolUnavailableException(name);
    }
    
    return await tool.execute(params);
  }
}
```

### 2.6 Memory System

#### 2.6.1 架构设计

```
┌─────────────────────────────────────────────────────────────────┐
│                        Memory System                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    MemoryManager                         │   │
│  ├─────────────────────────────────────────────────────────┤   │
│  │ + store(memory: Memory): Future<void>                   │   │
│  │ + search(query: String, limit: int): Future<List>       │   │
│  │ + get(memoryId: String): Future<Memory?>                │   │
│  │ + update(memoryId: String, updates: Map): Future<void>  │   │
│  │ + delete(memoryId: String): Future<void>                │   │
│  │ + getRelevant(context: String): Future<List>            │   │
│  │ + summarize(): Future<MemorySummary>                    │   │
│  └─────────────────────────────────────────────────────────┘   │
│                              │                                  │
│         ┌────────────────────┼────────────────────┐            │
│         ▼                    ▼                    ▼            │
│  ┌─────────────┐      ┌─────────────┐      ┌─────────────┐    │
│  │   Vector    │      │   Storage   │      │   Context   │    │
│  │   Store     │      │   Layer     │      │   Builder   │    │
│  └──────┬──────┘      └──────┬──────┘      └──────┬──────┘    │
│         │                    │                    │            │
│         ▼                    ▼                    ▼            │
│  ┌─────────────┐      ┌─────────────┐      ┌─────────────┐    │
│  │  Embedding  │      │   SQLite    │      │   Memory    │    │
│  │   Model     │      │   Storage   │      │  Compressor │    │
│  └─────────────┘      └─────────────┘      └─────────────┘    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

#### 2.6.2 核心实现

```dart
class MemoryManager {
  final VectorStore _vectorStore;
  final MemoryStorage _storage;
  final EmbeddingModel _embeddingModel;
  final MemoryCompressor _compressor;
  
  static const int _maxShortTermMemory = 10;
  static const int _maxLongTermMemory = 1000;

  MemoryManager({
    required VectorStore vectorStore,
    required MemoryStorage storage,
    required EmbeddingModel embeddingModel,
    required MemoryCompressor compressor,
  })  : _vectorStore = vectorStore,
        _storage = storage,
        _embeddingModel = embeddingModel,
        _compressor = compressor;

  Future<void> store(Memory memory) async {
    final embedding = await _embeddingModel.embed(memory.content);
    memory.embedding = embedding;
    
    await _storage.save(memory);
    await _vectorStore.insert(memory.id, embedding, metadata: {
      'type': memory.type.name,
      'importance': memory.importance,
      'createdAt': memory.createdAt.millisecondsSinceEpoch,
    });
    
    await _checkAndCompress();
  }

  Future<List<Memory>> search(String query, {int limit = 5}) async {
    final queryEmbedding = await _embeddingModel.embed(query);
    
    final results = await _vectorStore.search(
      queryEmbedding,
      limit: limit,
    );
    
    final memories = <Memory>[];
    for (final result in results) {
      final memory = await _storage.get(result.id);
      if (memory != null) {
        memory.relevanceScore = result.score;
        memories.add(memory);
      }
    }
    
    return memories;
  }

  Future<List<Memory>> getRelevant(String context) async {
    final recentMemories = await _storage.getRecent(limit: _maxShortTermMemory);
    final relevantMemories = await search(context, limit: 5);
    
    final allMemories = <Memory>[
      ...recentMemories,
      ...relevantMemories,
    ];
    
    final uniqueMemories = {for (var m in allMemories) m.id: m}.values.toList();
    uniqueMemories.sort((a, b) => b.importance.compareTo(a.importance));
    
    return uniqueMemories.take(10).toList();
  }

  Future<MemorySummary> summarize() async {
    final allMemories = await _storage.getAll();
    
    final byType = <MemoryType, List<Memory>>{};
    for (final memory in allMemories) {
      byType.putIfAbsent(memory.type, () => []).add(memory);
    }
    
    final summaries = <String, String>{};
    for (final entry in byType.entries) {
      final content = entry.value.map((m) => m.content).join('\n');
      summaries[entry.key.name] = await _compressor.compress(content);
    }
    
    return MemorySummary(
      totalMemories: allMemories.length,
      byType: byType.map((k, v) => MapEntry(k.name, v.length)),
      summaries: summaries,
    );
  }

  Future<void> _checkAndCompress() async {
    final count = await _storage.count();
    
    if (count > _maxLongTermMemory) {
      final oldMemories = await _storage.getOldest(
        limit: count - _maxLongTermMemory,
      );
      
      for (final memory in oldMemories) {
        if (memory.importance < 0.5) {
          await _storage.delete(memory.id);
          await _vectorStore.delete(memory.id);
        }
      }
    }
  }
}

enum MemoryType {
  conversation,
  userPreference,
  deviceState,
  environment,
  emotion,
  custom,
}

class Memory {
  final String id;
  final String content;
  final MemoryType type;
  final double importance;
  final DateTime createdAt;
  final DateTime? expiresAt;
  final Map<String, dynamic> metadata;
  
  List<double>? embedding;
  double? relevanceScore;
  
  Memory({
    required this.id,
    required this.content,
    required this.type,
    this.importance = 0.5,
    DateTime? createdAt,
    this.expiresAt,
    this.metadata = const {},
    this.embedding,
  }) : createdAt = createdAt ?? DateTime.now();
}
```

### 2.7 User Profile Manager

#### 2.7.1 架构设计

```dart
class UserProfileManager {
  final ProfileStorage _storage;
  final BehaviorAnalyzer _behaviorAnalyzer;
  final PreferenceLearner _preferenceLearner;
  
  UserProfile? _currentProfile;

  UserProfileManager({
    required ProfileStorage storage,
    required BehaviorAnalyzer behaviorAnalyzer,
    required PreferenceLearner preferenceLearner,
  })  : _storage = storage,
        _behaviorAnalyzer = behaviorAnalyzer,
        _preferenceLearner = preferenceLearner;

  Future<UserProfile> getProfile() async {
    _currentProfile ??= await _storage.load();
    return _currentProfile!;
  }

  Future<void> updateProfile(UserProfileUpdate update) async {
    final profile = await getProfile();
    
    switch (update.type) {
      case UpdateType.temperaturePreference:
        _updateTemperaturePreference(profile, update.data);
        break;
      case UpdateType.entertainmentPreference:
        _updateEntertainmentPreference(profile, update.data);
        break;
      case UpdateType.habit:
        _updateHabit(profile, update.data);
        break;
      case UpdateType.emotionalPattern:
        _updateEmotionalPattern(profile, update.data);
        break;
    }
    
    profile.lastUpdated = DateTime.now();
    await _storage.save(profile);
  }

  Future<void> learnFromInteraction(Interaction interaction) async {
    final profile = await getProfile();
    
    final behaviorPatterns = _behaviorAnalyzer.analyze(interaction);
    for (final pattern in behaviorPatterns) {
      await _updatePattern(profile, pattern);
    }
    
    final preferences = _preferenceLearner.learn(interaction, profile);
    for (final pref in preferences) {
      await _updatePreference(profile, pref);
    }
    
    await _storage.save(profile);
  }

  Future<RecommendationContext> buildRecommendationContext() async {
    final profile = await getProfile();
    final now = DateTime.now();
    
    return RecommendationContext(
      currentTime: now,
      season: _getSeason(now),
      dayOfWeek: now.weekday,
      timeOfDay: _getTimeOfDay(now),
      temperaturePreference: _getApplicableTemperaturePreference(profile, now),
      entertainmentPreference: profile.preferences.entertainment,
      lightingPreference: _getApplicableLightingPreference(profile, now),
      recentHabits: _getRecentHabits(profile),
      emotionalState: await _inferEmotionalState(profile),
    );
  }

  void _updateTemperaturePreference(UserProfile profile, Map<String, dynamic> data) {
    final season = data['season'] as String;
    final temperature = data['temperature'] as int;
    
    final tempPref = profile.preferences.temperature;
    if (season == 'summer') {
      tempPref.summer.preferred = temperature;
      if (temperature < tempPref.summer.min) tempPref.summer.min = temperature;
      if (temperature > tempPref.summer.max) tempPref.summer.max = temperature;
    } else {
      tempPref.winter.preferred = temperature;
      if (temperature < tempPref.winter.min) tempPref.winter.min = temperature;
      if (temperature > tempPref.winter.max) tempPref.winter.max = temperature;
    }
  }
}

class UserProfile {
  final String userId;
  final BasicInfo basicInfo;
  final Preferences preferences;
  final Habits habits;
  final EmotionalPatterns emotionalPatterns;
  final List<CustomScene> customScenes;
  DateTime lastUpdated;
  
  UserProfile({
    required this.userId,
    required this.basicInfo,
    required this.preferences,
    required this.habits,
    required this.emotionalPatterns,
    required this.customScenes,
    required this.lastUpdated,
  });
}

class Preferences {
  TemperaturePreference temperature;
  EntertainmentPreference entertainment;
  LightingPreference lighting;
  
  Preferences({
    required this.temperature,
    required this.entertainment,
    required this.lighting,
  });
}

class TemperaturePreference {
  SeasonRange summer;
  SeasonRange winter;
  
  TemperaturePreference({
    required this.summer,
    required this.winter,
  });
}

class SeasonRange {
  int min;
  int max;
  int preferred;
  
  SeasonRange({
    required this.min,
    required this.max,
    required this.preferred,
  });
}
```

---

## 3. 数据库设计

### 3.1 数据库架构

```
┌─────────────────────────────────────────────────────────────────┐
│                      Database Schema                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                     devices                              │   │
│  ├─────────────────────────────────────────────────────────┤   │
│  │ id              TEXT PRIMARY KEY                        │   │
│  │ name            TEXT NOT NULL                           │   │
│  │ type            TEXT NOT NULL                           │   │
│  │ protocol        TEXT NOT NULL                           │   │
│  │ manufacturer    TEXT                                    │   │
│  │ model           TEXT                                    │   │
│  │ firmware_version TEXT                                   │   │
│  │ connection_state TEXT NOT NULL                          │   │
│  │ current_state   TEXT                                    │   │
│  │ last_seen       INTEGER                                 │   │
│  │ last_connected  INTEGER                                 │   │
│  │ config          TEXT                                    │   │
│  │ capabilities    TEXT                                    │   │
│  │ room_id         TEXT                                    │   │
│  │ created_at      INTEGER NOT NULL                        │   │
│  │ updated_at      INTEGER NOT NULL                        │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                   conversations                          │   │
│  ├─────────────────────────────────────────────────────────┤   │
│  │ id              TEXT PRIMARY KEY                        │   │
│  │ title           TEXT                                    │   │
│  │ created_at      INTEGER NOT NULL                        │   │
│  │ updated_at      INTEGER NOT NULL                        │   │
│  │ message_count   INTEGER DEFAULT 0                       │   │
│  │ summary         TEXT                                    │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                     messages                             │   │
│  ├─────────────────────────────────────────────────────────┤   │
│  │ id              TEXT PRIMARY KEY                        │   │
│  │ conversation_id TEXT NOT NULL                           │   │
│  │ role            TEXT NOT NULL                           │   │
│  │ content         TEXT NOT NULL                           │   │
│  │ timestamp       INTEGER NOT NULL                        │   │
│  │ tool_calls      TEXT                                    │   │
│  │ tool_results    TEXT                                    │   │
│  │ metadata        TEXT                                    │   │
│  │ FOREIGN KEY (conversation_id) REFERENCES conversations  │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                     memories                             │   │
│  ├─────────────────────────────────────────────────────────┤   │
│  │ id              TEXT PRIMARY KEY                        │   │
│  │ content         TEXT NOT NULL                           │   │
│  │ type            TEXT NOT NULL                           │   │
│  │ importance      REAL DEFAULT 0.5                        │   │
│  │ embedding       BLOB                                    │   │
│  │ created_at      INTEGER NOT NULL                        │   │
│  │ expires_at      INTEGER                                 │   │
│  │ metadata        TEXT                                    │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                      scenes                              │   │
│  ├─────────────────────────────────────────────────────────┤   │
│  │ id              TEXT PRIMARY KEY                        │   │
│  │ name            TEXT NOT NULL                           │   │
│  │ description     TEXT                                    │   │
│  │ icon            TEXT                                    │   │
│  │ triggers        TEXT NOT NULL                           │   │
│  │ actions         TEXT NOT NULL                           │   │
│  │ conditions      TEXT                                    │   │
│  │ schedule        TEXT                                    │   │
│  │ enabled         INTEGER DEFAULT 1                       │   │
│  │ last_executed   INTEGER                                 │   │
│  │ created_at      INTEGER NOT NULL                        │   │
│  │ updated_at      INTEGER NOT NULL                        │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                   user_profile                           │   │
│  ├─────────────────────────────────────────────────────────┤   │
│  │ id              TEXT PRIMARY KEY                        │   │
│  │ user_id         TEXT NOT NULL UNIQUE                    │   │
│  │ basic_info      TEXT NOT NULL                           │   │
│  │ preferences     TEXT NOT NULL                           │   │
│  │ habits          TEXT NOT NULL                           │   │
│  │ emotional_patterns TEXT NOT NULL                        │   │
│  │ custom_scenes   TEXT                                    │   │
│  │ last_updated    INTEGER NOT NULL                        │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    settings                              │   │
│  ├─────────────────────────────────────────────────────────┤   │
│  │ key             TEXT PRIMARY KEY                        │   │
│  │ value           TEXT NOT NULL                           │   │
│  │ encrypted       INTEGER DEFAULT 0                       │   │
│  │ updated_at      INTEGER NOT NULL                        │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 3.2 数据访问层实现

```dart
class DatabaseService {
  static Database? _database;
  static const String _databaseName = 'mobile_claw.db';
  static const int _databaseVersion = 1;
  
  Future<Database> get database async {
    _database ??= await _initDatabase();
    return _database!;
  }
  
  Future<Database> _initDatabase() async {
    final dbPath = await getDatabasesPath();
    final path = join(dbPath, _databaseName);
    
    return await openDatabase(
      path,
      version: _databaseVersion,
      onCreate: _onCreate,
      onUpgrade: _onUpgrade,
    );
  }
  
  Future<void> _onCreate(Database db, int version) async {
    await db.execute('''
      CREATE TABLE devices (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        type TEXT NOT NULL,
        protocol TEXT NOT NULL,
        manufacturer TEXT,
        model TEXT,
        firmware_version TEXT,
        connection_state TEXT NOT NULL,
        current_state TEXT,
        last_seen INTEGER,
        last_connected INTEGER,
        config TEXT,
        capabilities TEXT,
        room_id TEXT,
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL
      )
    ''');
    
    await db.execute('''
      CREATE TABLE conversations (
        id TEXT PRIMARY KEY,
        title TEXT,
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL,
        message_count INTEGER DEFAULT 0,
        summary TEXT
      )
    ''');
    
    await db.execute('''
      CREATE TABLE messages (
        id TEXT PRIMARY KEY,
        conversation_id TEXT NOT NULL,
        role TEXT NOT NULL,
        content TEXT NOT NULL,
        timestamp INTEGER NOT NULL,
        tool_calls TEXT,
        tool_results TEXT,
        metadata TEXT,
        FOREIGN KEY (conversation_id) REFERENCES conversations (id)
      )
    ''');
    
    await db.execute('''
      CREATE TABLE memories (
        id TEXT PRIMARY KEY,
        content TEXT NOT NULL,
        type TEXT NOT NULL,
        importance REAL DEFAULT 0.5,
        embedding BLOB,
        created_at INTEGER NOT NULL,
        expires_at INTEGER,
        metadata TEXT
      )
    ''');
    
    await db.execute('''
      CREATE TABLE scenes (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        description TEXT,
        icon TEXT,
        triggers TEXT NOT NULL,
        actions TEXT NOT NULL,
        conditions TEXT,
        schedule TEXT,
        enabled INTEGER DEFAULT 1,
        last_executed INTEGER,
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL
      )
    ''');
    
    await db.execute('''
      CREATE TABLE user_profile (
        id TEXT PRIMARY KEY,
        user_id TEXT NOT NULL UNIQUE,
        basic_info TEXT NOT NULL,
        preferences TEXT NOT NULL,
        habits TEXT NOT NULL,
        emotional_patterns TEXT NOT NULL,
        custom_scenes TEXT,
        last_updated INTEGER NOT NULL
      )
    ''');
    
    await db.execute('''
      CREATE TABLE settings (
        key TEXT PRIMARY KEY,
        value TEXT NOT NULL,
        encrypted INTEGER DEFAULT 0,
        updated_at INTEGER NOT NULL
      )
    ''');
    
    await _createIndexes(db);
  }
  
  Future<void> _createIndexes(Database db) async {
    await db.execute('CREATE INDEX idx_messages_conversation ON messages (conversation_id)');
    await db.execute('CREATE INDEX idx_messages_timestamp ON messages (timestamp)');
    await db.execute('CREATE INDEX idx_memories_type ON memories (type)');
    await db.execute('CREATE INDEX idx_memories_created ON memories (created_at)');
    await db.execute('CREATE INDEX idx_devices_type ON devices (type)');
    await db.execute('CREATE INDEX idx_devices_room ON devices (room_id)');
  }
  
  Future<void> _onUpgrade(Database db, int oldVersion, int newVersion) async {
    // Handle database migrations
  }
}
```

---

## 4. 安全设计

### 4.1 安全架构

```
┌─────────────────────────────────────────────────────────────────┐
│                      Security Architecture                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                   Security Manager                       │   │
│  ├─────────────────────────────────────────────────────────┤   │
│  │ + encrypt(data: String): Future<String>                 │   │
│  │ + decrypt(encrypted: String): Future<String>            │   │
│  │ + hash(data: String): String                            │   │
│  │ + verifyHash(data: String, hash: String): bool          │   │
│  │ + generateKey(): Future<String>                         │   │
│  │ + storeSecure(key: String, value: String): Future<void> │   │
│  │ + retrieveSecure(key: String): Future<String?>          │   │
│  └─────────────────────────────────────────────────────────┘   │
│                              │                                  │
│         ┌────────────────────┼────────────────────┐            │
│         ▼                    ▼                    ▼            │
│  ┌─────────────┐      ┌─────────────┐      ┌─────────────┐    │
│  │   Data      │      │    Auth     │      │   Network   │    │
│  │ Encryption  │      │   Manager   │      │   Security  │    │
│  └──────┬──────┘      └──────┬──────┘      └──────┬──────┘    │
│         │                    │                    │            │
│         ▼                    ▼                    ▼            │
│  ┌─────────────┐      ┌─────────────┐      ┌─────────────┐    │
│  │  Keychain/  │      │    Token    │      │    TLS      │    │
│  │  Keystore   │      │   Manager   │      │   1.3       │    │
│  └─────────────┘      └─────────────┘      └─────────────┘    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 4.2 安全实现

```dart
class SecurityManager {
  final FlutterSecureStorage _secureStorage;
  final CryptographyService _crypto;
  
  static const String _keyPrefix = 'mc_secure_';
  static const String _masterKeyAlias = 'master_key';

  SecurityManager({
    required FlutterSecureStorage secureStorage,
    required CryptographyService crypto,
  })  : _secureStorage = secureStorage,
        _crypto = crypto;

  Future<void> initialize() async {
    final masterKey = await _secureStorage.read(key: _masterKeyAlias);
    if (masterKey == null) {
      await _generateAndStoreMasterKey();
    }
  }

  Future<String> encrypt(String data) async {
    final masterKey = await _getMasterKey();
    return await _crypto.encrypt(data, masterKey);
  }

  Future<String> decrypt(String encryptedData) async {
    final masterKey = await _getMasterKey();
    return await _crypto.decrypt(encryptedData, masterKey);
  }

  String hash(String data) {
    return _crypto.hash(data);
  }

  bool verifyHash(String data, String hash) {
    return _crypto.verifyHash(data, hash);
  }

  Future<void> storeSecure(String key, String value) async {
    final encrypted = await encrypt(value);
    await _secureStorage.write(key: '$_keyPrefix$key', value: encrypted);
  }

  Future<String?> retrieveSecure(String key) async {
    final encrypted = await _secureStorage.read(key: '$_keyPrefix$key');
    if (encrypted == null) return null;
    return await decrypt(encrypted);
  }

  Future<void> deleteSecure(String key) async {
    await _secureStorage.delete(key: '$_keyPrefix$key');
  }

  Future<String> _getMasterKey() async {
    return (await _secureStorage.read(key: _masterKeyAlias))!;
  }

  Future<void> _generateAndStoreMasterKey() async {
    final key = await _crypto.generateKey();
    await _secureStorage.write(key: _masterKeyAlias, value: key);
  }
}

class CryptographyService {
  final AesGcm _aesGcm;

  CryptographyService() : _aesGcm = AesGcm.with256bits();

  Future<String> encrypt(String plaintext, String key) async {
    final keyBytes = _deriveKey(key);
    final nonce = _aesGcm.newNonce();
    final secretBox = await _aesGcm.encrypt(
      utf8.encode(plaintext),
      secretKey: SecretKey(keyBytes),
      nonce: nonce,
    );
    return base64Encode(nonce + secretBox.mac.bytes + secretBox.cipherText);
  }

  Future<String> decrypt(String ciphertext, String key) async {
    final keyBytes = _deriveKey(key);
    final data = base64Decode(ciphertext);
    final nonce = data.sublist(0, 12);
    final mac = Mac(data.sublist(12, 28));
    final cipherText = data.sublist(28);
    
    final secretBox = SecretBox(
      cipherText,
      mac: mac,
      nonce: nonce,
    );
    
    final plaintext = await _aesGcm.decrypt(
      secretBox,
      secretKey: SecretKey(keyBytes),
    );
    return utf8.decode(plaintext);
  }

  String hash(String data) {
    final bytes = utf8.encode(data);
    final digest = sha256.convert(bytes);
    return digest.toString();
  }

  bool verifyHash(String data, String hash) {
    return this.hash(data) == hash;
  }

  Future<String> generateKey() async {
    final random = Random.secure();
    final bytes = List<int>.generate(32, (_) => random.nextInt(256));
    return base64Encode(bytes);
  }

  List<int> _deriveKey(String key) {
    return sha256.convert(utf8.encode(key)).bytes;
  }
}
```

---

## 5. 后台服务实现

### 5.1 Android 后台服务

```kotlin
class MobileClawService : Service() {
    private val NOTIFICATION_ID = 1001
    private val CHANNEL_ID = "mobile_claw_channel"
    
    private lateinit var agentCore: AgentCore
    private lateinit var deviceManager: DeviceManager
    private lateinit var networkGateway: NetworkGateway
    
    private var isRunning = false

    override fun onCreate() {
        super.onCreate()
        createNotificationChannel()
        startForeground(NOTIFICATION_ID, createNotification())
        
        initializeComponents()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        if (!isRunning) {
            startServices()
            isRunning = true
        }
        return START_STICKY
    }

    override fun onBind(intent: Intent?): IBinder? = null

    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                CHANNEL_ID,
                "Mobile Claw Service",
                NotificationManager.IMPORTANCE_LOW
            ).apply {
                description = "Mobile Claw background service"
                setShowBadge(false)
            }
            
            val manager = getSystemService(NotificationManager::class.java)
            manager.createNotificationChannel(channel)
        }
    }

    private fun createNotification(): Notification {
        return NotificationCompat.Builder(this, CHANNEL_ID)
            .setContentTitle("Mobile Claw")
            .setContentText("AI Agent is running")
            .setSmallIcon(R.drawable.ic_notification)
            .setPriority(NotificationCompat.PRIORITY_LOW)
            .setOngoing(true)
            .build()
    }

    private fun initializeComponents() {
        agentCore = AgentCore(applicationContext)
        deviceManager = DeviceManager(applicationContext)
        networkGateway = NetworkGateway(applicationContext)
    }

    private fun startServices() {
        CoroutineScope(Dispatchers.IO).launch {
            agentCore.start()
            deviceManager.start()
            networkGateway.start()
        }
    }

    override fun onDestroy() {
        super.onDestroy()
        agentCore.stop()
        deviceManager.stop()
        networkGateway.stop()
        
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            startForegroundService(Intent(this, MobileClawService::class.java))
        }
    }

    companion object {
        fun start(context: Context) {
            val intent = Intent(context, MobileClawService::class.java)
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                context.startForegroundService(intent)
            } else {
                context.startService(intent)
            }
        }

        fun stop(context: Context) {
            context.stopService(Intent(context, MobileClawService::class.java))
        }
    }
}

class BootReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        if (intent.action == Intent.ACTION_BOOT_COMPLETED) {
            val prefs = context.getSharedPreferences("mobile_claw", Context.MODE_PRIVATE)
            if (prefs.getBoolean("auto_start", true)) {
                MobileClawService.start(context)
            }
        }
    }
}
```

### 5.2 iOS 后台任务

```swift
class BackgroundTaskManager {
    static let shared = BackgroundTaskManager()
    
    private var backgroundTaskID: UIBackgroundTaskIdentifier = .invalid
    private var processingTask: BGProcessingTask?
    private var fetchTask: BGAppRefreshTask?
    
    private let agentCore: AgentCore
    private let deviceManager: DeviceManager
    private let networkGateway: NetworkGateway
    
    private init() {
        agentCore = AgentCore.shared
        deviceManager = DeviceManager.shared
        networkGateway = NetworkGateway.shared
        
        registerBackgroundTasks()
    }

    private func registerBackgroundTasks() {
        BGTaskScheduler.shared.register(
            forTaskWithIdentifier: "com.mobileclaw.processing",
            using: nil
        ) { task in
            self.handleProcessingTask(task as! BGProcessingTask)
        }
        
        BGTaskScheduler.shared.register(
            forTaskWithIdentifier: "com.mobileclaw.refresh",
            using: nil
        ) { task in
            self.handleRefreshTask(task as! BGAppRefreshTask)
        }
    }

    func scheduleBackgroundTasks() {
        let processingRequest = BGProcessingTaskRequest(identifier: "com.mobileclaw.processing")
        processingRequest.requiresNetworkConnectivity = false
        processingRequest.requiresExternalPower = false
        processingRequest.earliestBeginDate = Date(timeIntervalSinceNow: 15 * 60)
        
        do {
            try BGTaskScheduler.shared.submit(processingRequest)
        } catch {
            print("Could not schedule processing task: \(error)")
        }
        
        let refreshRequest = BGAppRefreshTaskRequest(identifier: "com.mobileclaw.refresh")
        refreshRequest.earliestBeginDate = Date(timeIntervalSinceNow: 30 * 60)
        
        do {
            try BGTaskScheduler.shared.submit(refreshRequest)
        } catch {
            print("Could not schedule refresh task: \(error)")
        }
    }

    private func handleProcessingTask(_ task: BGProcessingTask) {
        processingTask = task
        
        task.expirationHandler = {
            task.setTaskCompleted(success: false)
        }
        
        Task {
            do {
                try await agentCore.performBackgroundProcessing()
                try await deviceManager.syncDeviceStates()
                task.setTaskCompleted(success: true)
            } catch {
                task.setTaskCompleted(success: false)
            }
        }
    }

    private func handleRefreshTask(_ task: BGAppRefreshTask) {
        fetchTask = task
        
        task.expirationHandler = {
            task.setTaskCompleted(success: false)
        }
        
        Task {
            do {
                try await networkGateway.refreshConnections()
                try await deviceManager.checkDeviceAvailability()
                task.setTaskCompleted(success: true)
            } catch {
                task.setTaskCompleted(success: false)
            }
        }
    }

    func beginBackgroundTask() -> UIBackgroundTaskIdentifier {
        return UIApplication.shared.beginBackgroundTask {
            self.endBackgroundTask()
        }
    }

    func endBackgroundTask() {
        if backgroundTaskID != .invalid {
            UIApplication.shared.endBackgroundTask(backgroundTaskID)
            backgroundTaskID = .invalid
        }
    }
}
```

---

## 6. UI 层设计

### 6.1 状态管理架构

```
┌─────────────────────────────────────────────────────────────────┐
│                      State Management                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    Riverpod Providers                    │   │
│  ├─────────────────────────────────────────────────────────┤   │
│  │                                                         │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐    │   │
│  │  │ agentState  │  │ deviceState │  │  sceneState │    │   │
│  │  │  Provider   │  │  Provider   │  │  Provider   │    │   │
│  │  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘    │   │
│  │         │                │                │            │   │
│  │  ┌──────▼────────────────▼────────────────▼──────┐    │   │
│  │  │              appStateProvider                  │    │   │
│  │  └───────────────────────┬───────────────────────┘    │   │
│  │                          │                            │   │
│  │  ┌───────────────────────▼───────────────────────┐    │   │
│  │  │              Repository Layer                  │    │   │
│  │  │  - AgentRepository                             │    │   │
│  │  │  - DeviceRepository                            │    │   │
│  │  │  - SceneRepository                             │    │   │
│  │  │  - ProfileRepository                           │    │   │
│  │  └───────────────────────────────────────────────┘    │   │
│  │                                                         │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 6.2 状态管理实现

```dart
final agentStateProvider = StateNotifierProvider<AgentNotifier, AgentState>((ref) {
  return AgentNotifier(
    agentManager: ref.watch(agentManagerProvider),
  );
});

final deviceStateProvider = StateNotifierProvider<DeviceNotifier, DeviceState>((ref) {
  return DeviceNotifier(
    deviceManager: ref.watch(deviceManagerProvider),
  );
});

final sceneStateProvider = StateNotifierProvider<SceneNotifier, SceneState>((ref) {
  return SceneNotifier(
    sceneManager: ref.watch(sceneManagerProvider),
  );
});

class AgentNotifier extends StateNotifier<AgentState> {
  final AgentManager _agentManager;
  
  AgentNotifier({required AgentManager agentManager})
      : _agentManager = agentManager,
        super(const AgentState.initial()) {
    _initialize();
  }

  Future<void> _initialize() async {
    state = const AgentState.loading();
    try {
      await _agentManager.initialize(AgentConfig.defaultConfig());
      state = const AgentState.ready();
    } catch (e) {
      state = AgentState.error(e.toString());
    }
  }

  Future<void> sendMessage(String message) async {
    state = state.copyWith(isProcessing: true);
    
    try {
      final response = await _agentManager.sendMessage(message);
      state = state.copyWith(
        lastResponse: response,
        isProcessing: false,
      );
    } catch (e) {
      state = state.copyWith(
        error: e.toString(),
        isProcessing: false,
      );
    }
  }

  Stream<String> streamMessage(String message) {
    return _agentManager.streamMessage(message);
  }
}

class DeviceNotifier extends StateNotifier<DeviceState> {
  final DeviceManager _deviceManager;
  
  DeviceNotifier({required DeviceManager deviceManager})
      : _deviceManager = deviceManager,
        super(const DeviceState.initial()) {
    _initialize();
  }

  Future<void> _initialize() async {
    state = const DeviceState.loading();
    try {
      final devices = await _deviceManager.getConnectedDevices();
      state = DeviceState.ready(devices: devices);
    } catch (e) {
      state = DeviceState.error(e.toString());
    }
  }

  Future<void> discoverDevices() async {
    state = state.copyWith(isScanning: true);
    
    try {
      final devices = await _deviceManager.discoverDevices();
      state = state.copyWith(
        discoveredDevices: devices,
        isScanning: false,
      );
    } catch (e) {
      state = state.copyWith(
        error: e.toString(),
        isScanning: false,
      );
    }
  }

  Future<void> connectDevice(String deviceId) async {
    try {
      await _deviceManager.connectDevice(deviceId);
      final devices = await _deviceManager.getConnectedDevices();
      state = state.copyWith(devices: devices);
    } catch (e) {
      state = state.copyWith(error: e.toString());
    }
  }

  Future<void> executeCommand(String deviceId, DeviceCommand command) async {
    try {
      await _deviceManager.executeCommand(deviceId, command);
      final devices = await _deviceManager.getConnectedDevices();
      state = state.copyWith(devices: devices);
    } catch (e) {
      state = state.copyWith(error: e.toString());
    }
  }
}
```

### 6.3 主要 UI 组件

```dart
class ChatScreen extends ConsumerWidget {
  final TextEditingController _messageController = TextEditingController();
  
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final agentState = ref.watch(agentStateProvider);
    final messages = ref.watch(messageHistoryProvider);
    
    return Scaffold(
      appBar: AppBar(
        title: const Text('Mobile Claw'),
        actions: [
          IconButton(
            icon: const Icon(Icons.settings),
            onPressed: () => context.push('/settings'),
          ),
        ],
      ),
      body: Column(
        children: [
          Expanded(
            child: messages.isEmpty
                ? _buildEmptyState()
                : _buildMessageList(messages),
          ),
          _buildInputArea(context, ref, agentState),
        ],
      ),
    );
  }

  Widget _buildEmptyState() {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(Icons.chat_bubble_outline, size: 64, color: Colors.grey),
          SizedBox(height: 16),
          Text(
            '开始与 AI Agent 对话',
            style: TextStyle(fontSize: 18, color: Colors.grey),
          ),
          SizedBox(height: 8),
          Text(
            '我可以帮你控制智能设备、管理场景等',
            style: TextStyle(color: Colors.grey),
          ),
        ],
      ),
    );
  }

  Widget _buildMessageList(List<Message> messages) {
    return ListView.builder(
      padding: EdgeInsets.all(16),
      itemCount: messages.length,
      itemBuilder: (context, index) {
        final message = messages[index];
        return MessageBubble(message: message);
      },
    );
  }

  Widget _buildInputArea(BuildContext context, WidgetRef ref, AgentState state) {
    return Container(
      padding: EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Theme.of(context).cardColor,
        boxShadow: [
          BoxShadow(
            color: Colors.black12,
            blurRadius: 4,
            offset: Offset(0, -2),
          ),
        ],
      ),
      child: Row(
        children: [
          Expanded(
            child: TextField(
              controller: _messageController,
              decoration: InputDecoration(
                hintText: '输入消息...',
                border: OutlineInputBorder(
                  borderRadius: BorderRadius.circular(24),
                ),
                contentPadding: EdgeInsets.symmetric(horizontal: 16),
              ),
              onSubmitted: (_) => _sendMessage(ref),
            ),
          ),
          SizedBox(width: 8),
          IconButton(
            icon: Icon(Icons.mic),
            onPressed: () => _startVoiceInput(ref),
          ),
          SizedBox(width: 8),
          IconButton(
            icon: state.isProcessing
                ? CircularProgressIndicator()
                : Icon(Icons.send),
            onPressed: state.isProcessing ? null : () => _sendMessage(ref),
          ),
        ],
      ),
    );
  }

  void _sendMessage(WidgetRef ref) {
    final message = _messageController.text.trim();
    if (message.isEmpty) return;
    
    _messageController.clear();
    ref.read(agentStateProvider.notifier).sendMessage(message);
    ref.read(messageHistoryProvider.notifier).addMessage(
      Message(role: Role.user, content: message),
    );
  }

  void _startVoiceInput(WidgetRef ref) {
    // Implement voice input
  }
}
```

---

## 7. 性能优化

### 7.1 模型推理优化

```dart
class ModelOptimization {
  final ModelCache _cache;
  final MemoryManager _memoryManager;
  
  static const int _maxCacheSize = 100 * 1024 * 1024; // 100MB
  static const int _kvCacheSize = 2048; // tokens

  Future<void> optimizeForDevice() async {
    final deviceInfo = await _getDeviceInfo();
    
    // 根据设备性能调整参数
    if (deviceInfo.totalMemory < 4 * 1024 * 1024 * 1024) {
      await _applyLowMemoryOptimizations();
    } else if (deviceInfo.totalMemory < 6 * 1024 * 1024 * 1024) {
      await _applyMediumMemoryOptimizations();
    } else {
      await _applyHighMemoryOptimizations();
    }
    
    // GPU 加速检测
    if (deviceInfo.hasGpu) {
      await _enableGpuAcceleration();
    }
  }

  Future<void> _applyLowMemoryOptimizations() async {
    // 使用量化模型
    // 减少上下文长度
    // 启用 KV cache 压缩
  }

  Future<void> _applyMediumMemoryOptimizations() async {
    // 使用 INT4 量化
    // 中等上下文长度
  }

  Future<void> _applyHighMemoryOptimizations() async {
    // 使用 INT8 或 FP16
    // 完整上下文长度
  }

  Future<InferenceResult> optimizedInference(
    String prompt,
    InferenceConfig config,
  ) async {
    // 预处理优化
    final optimizedPrompt = _optimizePrompt(prompt);
    
    // 缓存检查
    final cacheKey = _generateCacheKey(optimizedPrompt);
    final cached = await _cache.get(cacheKey);
    if (cached != null) {
      return cached;
    }
    
    // 批处理优化
    if (_shouldBatch(config)) {
      return await _batchInference(optimizedPrompt, config);
    }
    
    // 流式推理
    return await _streamInference(optimizedPrompt, config);
  }

  String _optimizePrompt(String prompt) {
    // 移除冗余空白
    // 压缩重复内容
    // 优化 token 使用
    return prompt.trim();
  }

  bool _shouldBatch(InferenceConfig config) {
    return config.enableBatch && _pendingRequests.length >= 2;
  }
}
```

### 7.2 网络优化

```dart
class NetworkOptimization {
  final ConnectionPool _connectionPool;
  final RequestQueue _requestQueue;
  final CacheManager _cacheManager;
  
  static const int _maxConnections = 10;
  static const Duration _connectionTimeout = Duration(seconds: 30);
  static const Duration _requestTimeout = Duration(seconds: 60);

  Future<void> initialize() async {
    await _connectionPool.initialize(
      maxSize: _maxConnections,
      timeout: _connectionTimeout,
    );
    
    _startConnectionMonitor();
    _startCacheCleanup();
  }

  Future<Response> optimizedRequest(Request request) async {
    // 缓存检查
    if (request.method == 'GET') {
      final cached = await _cacheManager.get(request.cacheKey);
      if (cached != null && !cached.isExpired) {
        return cached.response;
      }
    }
    
    // 连接复用
    final connection = await _connectionPool.acquire();
    
    try {
      // 请求合并
      if (_canMerge(request)) {
        return await _mergeRequest(request);
      }
      
      // 执行请求
      final response = await connection.send(
        request,
        timeout: _requestTimeout,
      );
      
      // 缓存响应
      if (request.cacheable && response.isSuccessful) {
        await _cacheManager.set(
          request.cacheKey,
          CachedResponse(response, request.cacheDuration),
        );
      }
      
      return response;
    } finally {
      _connectionPool.release(connection);
    }
  }

  void _startConnectionMonitor() {
    Timer.periodic(Duration(seconds: 30), (_) {
      _connectionPool.cleanup();
      _checkConnectionHealth();
    });
  }

  Future<void> _checkConnectionHealth() async {
    final connections = _connectionPool.activeConnections;
    for (final conn in connections) {
      if (!await conn.isHealthy()) {
        _connectionPool.remove(conn);
      }
    }
  }
}
```

### 7.3 内存优化

```dart
class MemoryOptimizer {
  final MemoryMonitor _monitor;
  final GarbageCollector _gc;
  
  static const int _warningThreshold = 80; // 80% 内存使用
  static const int _criticalThreshold = 90; // 90% 内存使用

  void startMonitoring() {
    Timer.periodic(Duration(seconds: 10), (_) {
      _checkMemoryUsage();
    });
  }

  void _checkMemoryUsage() {
    final usage = _monitor.getCurrentUsage();
    final percentage = (usage.used / usage.total) * 100;
    
    if (percentage >= _criticalThreshold) {
      _handleCriticalMemory();
    } else if (percentage >= _warningThreshold) {
      _handleWarningMemory();
    }
  }

  void _handleWarningMemory() {
    // 清理非必要缓存
    _gc.clearNonEssentialCaches();
    
    // 压缩内存中的数据
    _gc.compressInMemoryData();
    
    // 通知用户
    _showMemoryWarning();
  }

  void _handleCriticalMemory() {
    // 释放模型内存
    _gc.unloadModel();
    
    // 清理所有缓存
    _gc.clearAllCaches();
    
    // 减少后台任务
    _gc.reduceBackgroundTasks();
    
    // 强制垃圾回收
    _gc.forceCollection();
  }

  Future<void> optimizeForBackground() async {
    // 减少内存占用
    await _gc.reduceMemoryFootprint();
    
    // 压缩数据结构
    await _gc.compressDataStructures();
    
    // 释放非必要资源
    await _gc.releaseNonEssentialResources();
  }
}
```

---

## 8. 测试策略

### 8.1 测试架构

```
┌─────────────────────────────────────────────────────────────────┐
│                       Testing Strategy                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                     E2E Tests                            │   │
│  │  - User Flow Testing                                    │   │
│  │  - Integration Scenarios                                │   │
│  │  - Performance Testing                                  │   │
│  └─────────────────────────────────────────────────────────┘   │
│                              △                                  │
│                              │                                  │
│  ┌───────────────────────────┴───────────────────────────┐     │
│  │                  Integration Tests                      │     │
│  │  - Module Integration                                  │     │
│  │  - API Integration                                     │     │
│  │  - Database Integration                                │     │
│  └─────────────────────────────────────────────────────────┘   │
│                              △                                  │
│                              │                                  │
│  ┌───────────────────────────┴───────────────────────────┐     │
│  │                    Unit Tests                           │     │
│  │  - Business Logic                                      │     │
│  │  - Utility Functions                                   │     │
│  │  - Data Models                                         │     │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 8.2 单元测试示例

```dart
group('AgentManager', () {
  late AgentManager agentManager;
  late MockModelRuntime mockModelRuntime;
  late MockToolRegistry mockToolRegistry;
  late MockMemorySystem mockMemorySystem;

  setUp(() {
    mockModelRuntime = MockModelRuntime();
    mockToolRegistry = MockToolRegistry();
    mockMemorySystem = MockMemorySystem();
    
    agentManager = AgentManagerImpl(
      modelRuntime: mockModelRuntime,
      toolRegistry: mockToolRegistry,
      memorySystem: mockMemorySystem,
      profileManager: MockProfileManager(),
      config: AgentConfig.defaultConfig(),
    );
  });

  test('should initialize successfully', () async {
    when(() => mockModelRuntime.initialize(any()))
        .thenAnswer((_) async => {});
    
    await agentManager.initialize(AgentConfig.defaultConfig());
    
    verify(() => mockModelRuntime.initialize(any())).called(1);
  });

  test('should send message and return response', () async {
    when(() => mockModelRuntime.runLocal(
      prompt: any(named: 'prompt'),
      config: any(named: 'config'),
    )).thenAnswer((_) async => InferenceResult(
      content: 'Hello! How can I help you?',
      fromCache: false,
      tokensGenerated: 10,
      inferenceTime: Duration(milliseconds: 100),
    ));

    await agentManager.initialize(AgentConfig.defaultConfig());
    final response = await agentManager.sendMessage('Hello');

    expect(response.message.content, contains('Hello'));
    expect(response.message.role, equals(Role.assistant));
  });

  test('should execute tool calls', () async {
    final toolCall = ToolCall(
      id: 'tc_1',
      name: 'ac_set_temperature',
      arguments: {'device_id': 'ac_1', 'temperature': 24},
    );

    when(() => mockModelRuntime.runLocal(
      prompt: any(named: 'prompt'),
      config: any(named: 'config'),
    )).thenAnswer((_) async => InferenceResult(
      content: '<tool_call name="ac_set_temperature">{"device_id":"ac_1","temperature":24}</tool_call
>',
      fromCache: false,
      tokensGenerated: 20,
      inferenceTime: Duration(milliseconds: 150),
      toolCalls: [toolCall],
    ));

    when(() => mockToolRegistry.executeTool(
      'ac_set_temperature',
      {'device_id': 'ac_1', 'temperature': 24},
    )).thenAnswer((_) async => ToolResult(
      success: true,
      output: {'temperature': 24},
    ));

    await agentManager.initialize(AgentConfig.defaultConfig());
    final response = await agentManager.sendMessage('Set AC to 24');

    expect(response.toolResults, isNotEmpty);
    expect(response.toolResults!.first.success, isTrue);
  });

  test('should handle errors gracefully', () async {
    when(() => mockModelRuntime.runLocal(
      prompt: any(named: 'prompt'),
      config: any(named: 'config'),
    )).thenThrow(ModelException('Model failed'));

    await agentManager.initialize(AgentConfig.defaultConfig());
    
    expect(
      () => agentManager.sendMessage('Hello'),
      throwsA(isA<AgentException>()),
    );
  });
});
```

### 8.3 集成测试示例

```dart
group('Device Integration Tests', () {
  testWidgets('should discover and connect to device', (tester) async {
    await tester.pumpWidget(MobileClawApp());
    
    // Navigate to devices page
    await tester.tap(find.iconButton(Icons.devices));
    await tester.pumpAndSettle();
    
    // Start discovery
    await tester.tap(find.text('Scan Devices'));
    await tester.pump(Duration(seconds: 2));
    
    // Verify device found
    expect(find.text('Living Room AC'), findsOneWidget);
    
    // Connect to device
    await tester.tap(find.text('Living Room AC'));
    await tester.pumpAndSettle();
    
    // Verify connected
    expect(find.text('Connected'), findsOneWidget);
  });

  testWidgets('should execute device command', (tester) async {
    await tester.pumpWidget(MobileClawApp());
    
    // Navigate to device control
    await _navigateToDevice(tester, 'Living Room AC');
    
    // Change temperature
    await tester.tap(find.byIcon(Icons.add));
    await tester.pumpAndSettle();
    
    // Verify temperature changed
    expect(find.text('25°C'), findsOneWidget);
  });
});
```

---

## 9. 部署方案

### 9.1 Android 部署

```gradle
android {
    compileSdk 34

    defaultConfig {
        applicationId "com.mobileclaw.app"
        minSdk 26
        targetSdk 34
        versionCode 1
        versionName "1.0.0"
        
        ndk {
            abiFilters 'armeabi-v7a', 'arm64-v8a', 'x86_64'
        }
    }

    buildTypes {
        release {
            minifyEnabled true
            shrinkResources true
            proguardFiles getDefaultProguardFile('proguard-android-optimize.txt'), 'proguard-rules.pro'
            signingConfig signingConfigs.release
        }
        
        debug {
            debuggable true
            applicationIdSuffix ".debug"
        }
    }

    packagingOptions {
        resources {
            excludes += ['/META-INF/{AL2.0,LGPL2.1}']
        }
    }
}
```

### 9.2 iOS 部署

```ruby
platform :ios, '14.0'

target 'MobileClaw' do
  use_frameworks!
  
  pod 'SQLite.swift', '~> 0.13.0'
  pod 'KeychainAccess', '~> 4.2'
  pod 'Alamofire', '~> 5.8'
end

post_install do |installer|
  installer.pods_project.targets.each do |target|
    target.build_configurations.each do |config|
      config.build_settings['IPHONEOS_DEPLOYMENT_TARGET'] = '14.0'
    end
  end
end
```

### 9.3 CI/CD 配置

```yaml
name: Mobile Claw CI/CD

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Flutter
        uses: subosito/flutter-action@v2
        with:
          flutter-version: '3.16.0'
      
      - name: Install dependencies
        run: flutter pub get
      
      - name: Run tests
        run: flutter test --coverage
      
      - name: Upload coverage
        uses: codecov/codecov-action@v3

  build-android:
    needs: test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Java
        uses: actions/setup-java@v3
        with:
          java-version: '17'
          distribution: 'temurin'
      
      - name: Setup Flutter
        uses: subosito/flutter-action@v2
        with:
          flutter-version: '3.16.0'
      
      - name: Build APK
        run: flutter build apk --release
      
      - name: Build App Bundle
        run: flutter build appbundle --release
      
      - name: Upload to Play Store
        uses: r0adkll/upload-google-play@v1
        with:
          serviceAccountJsonPlainText: ${{ secrets.PLAY_STORE_SERVICE_ACCOUNT }}
          packageName: com.mobileclaw.app
          releaseFiles: build/app/outputs/bundle/release/app-release.aab
          track: internal

  build-ios:
    needs: test
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Flutter
        uses: subosito/flutter-action@v2
        with:
          flutter-version: '3.16.0'
      
      - name: Build iOS
        run: flutter build ios --release --no-codesign
      
      - name: Archive and upload to TestFlight
        run: |
          xcodebuild -workspace ios/Runner.xcworkspace \
            -scheme Runner \
            -archivePath build/Runner.xcarchive \
            archive
          xcodebuild -exportArchive \
            -archivePath build/Runner.xcarchive \
            -exportOptionsPlist ios/ExportOptions.plist \
            -exportPath build/
```

---

## 10. 开发计划

### 10.1 里程碑规划

| 阶段 | 时间 | 主要目标 | 交付物 |
|-----|------|---------|--------|
| M1 | Week 1-2 | 项目初始化 | 项目架构、开发环境 |
| M2 | Week 3-4 | 核心框架 | Agent Manager、Device Manager |
| M3 | Week 5-6 | 后台服务 | Android Service、iOS Daemon |
| M4 | Week 7-8 | 本地模型 | 模型集成、推理优化 |
| M5 | Week 9-10 | 协议支持 | A2A、ACP、MCP |
| M6 | Week 11-12 | 设备控制 | 工具集、设备适配 |
| M7 | Week 13-14 | 智能能力 | 用户画像、推荐系统 |
| M8 | Week 15-16 | UI 完善 | 界面开发、交互优化 |
| M9 | Week 17-18 | 测试优化 | 测试、性能优化 |
| M10 | Week 19-20 | 发布准备 | 文档、应用商店提交 |

### 10.2 详细任务分解

**M1: 项目初始化 (Week 1-2)**
- [ ] 创建项目结构
- [ ] 配置开发环境
- [ ] 搭建 CI/CD 流程
- [ ] 编写技术文档

**M2: 核心框架 (Week 3-4)**
- [ ] 实现 Agent Manager
- [ ] 实现 Device Manager
- [ ] 实现基础数据存储
- [ ] 编写单元测试

**M3: 后台服务 (Week 5-6)**
- [ ] Android Foreground Service
- [ ] iOS Background Tasks
- [ ] 服务保活机制
- [ ] 状态同步

**M4: 本地模型 (Week 7-8)**
- [ ] llama.cpp 集成
- [ ] 模型下载管理
- [ ] 推理优化
- [ ] GPU 加速

**M5: 协议支持 (Week 9-10)**
- [ ] A2A 协议实现
- [ ] ACP 协议实现
- [ ] MCP 协议实现
- [ ] 协议测试

**M6: 设备控制 (Week 11-12)**
- [ ] 设备发现
- [ ] 连接管理
- [ ] 控制工具集
- [ ] 设备适配器

**M7: 智能能力 (Week 13-14)**
- [ ] 用户画像系统
- [ ] 记忆系统
- [ ] 推荐引擎
- [ ] 情绪感知

**M8: UI 完善 (Week 15-16)**
- [ ] 聊天界面
- [ ] 设备控制界面
- [ ] 场景管理界面
- [ ] 设置界面

**M9: 测试优化 (Week 17-18)**
- [ ] 单元测试完善
- [ ] 集成测试
- [ ] 性能优化
- [ ] Bug 修复

**M10: 发布准备 (Week 19-20)**
- [ ] 用户文档
- [ ] 应用商店材料
- [ ] 提交审核
- [ ] 发布上线

---

## 11. 风险评估与应对

### 11.1 技术风险

| 风险 | 概率 | 影响 | 应对措施 |
|-----|------|------|---------|
| 本地模型性能不足 | 中 | 高 | 提供云端备选、模型量化优化 |
| 后台服务被系统杀掉 | 中 | 高 | 多重保活机制、用户引导设置 |
| 设备兼容性问题 | 高 | 中 | 协议适配层、持续更新兼容库 |
| 内存溢出 | 中 | 高 | 内存监控、自动释放机制 |
| 蓝牙连接不稳定 | 中 | 中 | 重连机制、连接状态监控 |

### 11.2 项目风险

| 风险 | 概率 | 影响 | 应对措施 |
|-----|------|------|---------|
| 开发周期延长 | 中 | 中 | 分阶段交付、优先核心功能 |
| 需求变更 | 中 | 中 | 敏捷开发、快速迭代 |
| 人力资源不足 | 低 | 高 | 合理分配、必要时外包 |
| 应用商店审核不通过 | 低 | 高 | 提前了解规则、准备合规材料 |

### 11.3 安全风险

| 风险 | 概率 | 影响 | 应对措施 |
|-----|------|------|---------|
| 数据泄露 | 低 | 高 | 数据加密、权限控制 |
| API 密钥泄露 | 低 | 高 | 安全存储、不在日志输出 |
| 恶意设备接入 | 中 | 中 | 设备认证、安全协议 |
| 越权访问 | 低 | 中 | 权限管理、最小权限原则 |

---

## 12. 附录

### 12.1 术语表

| 术语 | 定义 |
|-----|------|
| A2A | Agent-to-Agent，Agent 间通信协议 |
| ACP | Agent Communication Protocol，Agent 通信协议 |
| MCP | Model Context Protocol，模型上下文协议 |
| BLE | Bluetooth Low Energy，低功耗蓝牙 |
| mDNS | Multicast DNS，多播 DNS |
| KV Cache | Key-Value Cache，键值缓存 |
| FFI | Foreign Function Interface，外部函数接口 |

### 12.2 参考资源

- Flutter 官方文档: https://flutter.dev/docs
- llama.cpp: https://github.com/ggerganov/llama.cpp
- Android Background Services: https://developer.android.com/guide/components/services
- iOS Background Tasks: https://developer.apple.com/documentation/backgroundtasks
- MCP 协议规范: https://modelcontextprotocol.io/

### 12.3 版本历史

| 版本 | 日期 | 作者 | 变更说明 |
|-----|------|------|---------|
| 1.0.0 | 2026-03-11 | ZeroClaw Team | 初始版本 |