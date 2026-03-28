---
title: ASR 配置测试连接按钮
type: feat
status: active
date: 2026-03-27
origin: docs/brainstorms/2026-03-27-asr-test-connection-requirements.md
---

# ASR 配置测试连接按钮

## Overview

在 ASR 配置面板添加"测试连接"按钮，让用户在保存配置前就能验证豆包或阿里云的 API Key 是否有效。

## Problem Frame

用户在配置 ASR（语音识别）服务时，需要填写豆包或阿里云的 API Key。目前用户只能在实际使用时才知道配置是否正确，如果配置错误会导致语音识别失败，体验不佳。需要在配置界面添加"测试连接"按钮，让用户在保存配置前就能验证 API Key 是否有效。

## Requirements Trace

- R1. 在 ASR 配置面板添加"测试连接"按钮，位置与 LLM 配置面板的测试按钮保持一致
- R2. 测试按钮应根据当前选中的 ASR 提供商（豆包/阿里云）测试对应的配置
- R3. 点击测试按钮后，应尝试建立与 ASR 服务的真实 WebSocket 连接
- R4. 测试结果应显示在按钮下方：成功显示绿色"连接成功"，失败显示红色错误信息
- R5. 测试过程中按钮应禁用，显示"测试中..."
- R6. 测试应在 10 秒内超时

## Scope Boundaries

- 仅测试连接建立和认证，不发送实际音频数据
- 不支持测试语音识别准确性
- 如果用户未填写 API Key，直接提示"请先填写 API Key"

## Context & Research

### Relevant Code and Patterns

**LLM 测试参考实现** (`KoeApp/Koe/SetupWizard/SPSetupWizardWindowController.m:1182-1257`):
- 使用 `NSURLSession` 直接发起 HTTP POST 请求
- 禁用按钮并显示"Testing..."
- 使用 `dispatch_async(dispatch_get_main_queue())` 更新 UI
- 成功显示绿色文字，失败显示红色文字

**ASR 配置面板** (`SPSetupWizardWindowController.m:451-533`):
- Provider 选择器: `asrProviderPopup`
- 豆包字段: `asrAppKeyField`, `asrAccessKeySecureField`
- 阿里云字段: `asrAliyunApiKeySecureField`
- Provider 切换通过 `asrProviderChanged:` 方法处理

**ASR 提供商连接参数**:
- 豆包: WebSocket `wss://openspeech.bytedance.com/api/v3/sauc/bigmodel_async`
  - Headers: `X-Api-App-Key`, `X-Api-Access-Key`, `X-Api-Resource-Id`
- 阿里云: WebSocket `wss://dashscope.aliyuncs.com/api-ws/v1/realtime?model=qwen3-asr-flash-realtime`
  - Header: `Authorization: Bearer {api_key}`

### External References

- [NSURLSessionWebSocketTask Documentation](https://developer.apple.com/documentation/foundation/nsurlsessionwebsockettask) - iOS 13.0+ / macOS 10.15+

## Key Technical Decisions

- **实现方案**: 在 Obj-C 层直接实现（与 LLM 测试保持一致）
  - 使用 `NSURLSessionWebSocketTask` 进行 WebSocket 连接测试
  - 无需修改 Rust 代码，降低复杂度
  - 与现有 LLM 测试代码风格一致

- **协议处理**:
  - 豆包: 连接后等待服务器的二进制响应或错误帧
  - 阿里云: 连接后等待 `session.created` JSON 事件

- **超时处理**: 10 秒超时，使用 `NSURLSession` 的 `timeoutIntervalForRequest`

- **测试深度**: 仅验证 WebSocket 连接可以建立并收到服务器响应，不发送音频数据

## Implementation Units

- [ ] **Unit 1: 添加 UI 控件**

**Goal:** 在 ASR 配置面板添加测试按钮和结果标签

**Requirements:** R1

**Dependencies:** None

**Files:**
- Modify: `KoeApp/Koe/SetupWizard/SPSetupWizardWindowController.m`
  - 在 `buildAsrPane` 方法中添加 `asrTestButton` 和 `asrTestResultLabel`
  - 位置参考 LLM 面板的实现（底部按钮上方）

**Approach:**
- 参考 `buildLlmPane` 中 `llmTestButton` 和 `llmTestResultLabel` 的实现
- 在 ASR 面板的 Save/Cancel 按钮上方添加测试按钮和结果标签
- 调整面板高度以容纳新控件

**Patterns to follow:**
- `buildLlmPane` 中测试按钮的布局和样式
- `llmTestButton` 和 `llmTestResultLabel` 的属性定义

**Test scenarios:**
- 打开 ASR 面板应看到"测试连接"按钮
- 按钮位置应在输入字段下方、保存按钮上方

**Verification:**
- 运行应用，打开设置 → ASR 面板，能看到测试按钮

---

- [ ] **Unit 2: 实现豆包连接测试**

**Goal:** 实现豆包 ASR 的 WebSocket 连接测试

**Requirements:** R2, R3, R4, R5, R6

**Dependencies:** Unit 1

**Files:**
- Modify: `KoeApp/Koe/SetupWizard/SPSetupWizardWindowController.m`
  - 添加 `testDoubaoConnection:` 方法
  - 添加 `asrTestButton` 和 `asrTestResultLabel` 属性声明

**Approach:**
- 使用 `NSURLSessionWebSocketTask` 建立 WebSocket 连接
- 设置请求头：`X-Api-App-Key`, `X-Api-Access-Key`, `X-Api-Resource-Id`
- 连接成功后立即关闭（不发送音频数据）
- 处理连接错误（认证失败、网络错误等）

**Technical design:**
```
1. 检查 app_key 和 access_key 是否已填写
2. 创建 NSURLSessionWebSocketTask
3. 设置 10 秒超时
4. 发送 WebSocket 连接请求（带认证头）
5. 等待连接建立
6. 收到任何服务器响应即视为成功
7. 关闭连接并更新 UI
```

**Patterns to follow:**
- `testLlmConnection:` 中的 UI 状态管理（禁用按钮、显示"测试中..."）
- `dispatch_async(dispatch_get_main_queue())` 更新 UI

**Test scenarios:**
- 填写正确的豆包 App Key 和 Access Key → 显示"连接成功"（绿色）
- 填写错误的 Key → 显示错误信息（红色）
- 空 Key → 提示"请先填写 API Key"
- 网络异常 → 显示网络错误
- 测试过程中按钮应禁用

**Verification:**
- 使用有效的豆包凭证测试显示成功
- 使用无效凭证测试显示失败

---

- [ ] **Unit 3: 实现阿里云连接测试**

**Goal:** 实现阿里云 ASR 的 WebSocket 连接测试

**Requirements:** R2, R3, R4, R5, R6

**Dependencies:** Unit 2

**Files:**
- Modify: `KoeApp/Koe/SetupWizard/SPSetupWizardWindowController.m`
  - 添加 `testAliyunConnection:` 方法

**Approach:**
- 使用 `NSURLSessionWebSocketTask` 建立 WebSocket 连接
- 设置请求头：`Authorization: Bearer {api_key}`
- 阿里云协议：连接后等待 `session.created` 事件
- 收到 `session.created` 或任何服务器响应即视为成功

**Technical design:**
```
1. 检查 api_key 是否已填写
2. 创建 NSURLSessionWebSocketTask
3. 设置 Authorization: Bearer {api_key} 头
4. 连接后等待服务器消息
5. 解析收到的 JSON 消息
6. 收到 session.created 或任何响应即视为成功
7. 关闭连接并更新 UI
```

**Patterns to follow:**
- 与豆包测试相同的 UI 状态管理
- 阿里云协议处理参考 `koe-asr/src/aliyun.rs:97-105`

**Test scenarios:**
- 填写正确的阿里云 API Key → 显示"连接成功"
- 填写错误的 Key → 显示认证失败错误
- 空 Key → 提示"请先填写 API Key"
- 测试过程中按钮应禁用

**Verification:**
- 使用有效的阿里云凭证测试显示成功
- 使用无效凭证测试显示失败

---

- [ ] **Unit 4: 添加主测试入口和 Provider 分发**

**Goal:** 根据当前选中的 Provider 调用对应的测试方法

**Requirements:** R2

**Dependencies:** Unit 2, Unit 3

**Files:**
- Modify: `KoeApp/Koe/SetupWizard/SPSetupWizardWindowController.m`
  - 添加 `testAsrConnection:` 方法

**Approach:**
- 创建统一的测试入口方法 `testAsrConnection:`
- 根据 `asrProviderPopup.selectedItem` 判断当前 Provider
- 调用对应的测试方法（豆包或阿里云）

**Technical design:**
```
- (void)testAsrConnection:(id)sender {
    NSString *provider = self.asrProviderPopup.selectedItem.representedObject;
    if ([provider isEqualToString:@"doubao"]) {
        [self testDoubaoConnection:sender];
    } else if ([provider isEqualToString:@"aliyun"]) {
        [self testAliyunConnection:sender];
    }
}
```

**Patterns to follow:**
- `asrProviderChanged:` 中获取 Provider 的方式

**Test scenarios:**
- 切换 Provider 后点击测试按钮应测试对应的配置
- Provider 切换时清空之前的测试结果

**Verification:**
- 选择豆包时测试豆包配置
- 选择阿里云时测试阿里云配置

---

- [ ] **Unit 5: UI 细节完善**

**Goal:** 完善 UI 交互细节，确保与 LLM 测试体验一致

**Requirements:** R4, R5

**Dependencies:** Unit 4

**Files:**
- Modify: `KoeApp/Koe/SetupWizard/SPSetupWizardWindowController.m`
  - `loadValuesForPane:` 方法中清空测试结果
  - `asrProviderChanged:` 方法中清空测试结果

**Approach:**
- 切换 Provider 时清空测试结果
- 加载配置时清空测试结果
- 确保按钮样式与 LLM 测试按钮一致

**Test scenarios:**
- 切换 Provider 后测试结果应清空
- 重新打开设置窗口后测试结果应清空
- 按钮样式应与 LLM 测试按钮一致

**Verification:**
- UI 交互符合预期，无残留状态

## System-Wide Impact

- **Interaction graph:** 仅影响设置窗口的 ASR 面板，无其他回调或观察者
- **Error propagation:** 所有错误都在 UI 层处理，不传播到核心功能
- **State lifecycle risks:** 测试连接在独立 WebSocket 中完成，不影响现有 ASR 会话
- **API surface parity:** 此功能仅在设置窗口使用，不影响运行时 API

## Risks & Dependencies

- **风险**: WebSocket 连接可能在某些网络环境下失败（如代理、防火墙）
  - 缓解: 提供清晰的错误信息，让用户知道是网络问题
- **风险**: 豆包或阿里云 API 变更可能导致测试失效
  - 缓解: 测试仅验证连接建立，不依赖具体响应格式

## Deferred to Implementation

- 豆包 ASR 是否支持仅建立连接：假设连接后立即收到服务器响应即视为成功
- 错误信息本地化：使用中文错误提示（与现有 UI 语言一致）

## Sources & References

- **Origin document:** [docs/brainstorms/2026-03-27-asr-test-connection-requirements.md](../../brainstorms/2026-03-27-asr-test-connection-requirements.md)
- **LLM 测试参考:** `KoeApp/Koe/SetupWizard/SPSetupWizardWindowController.m:1182-1257`
- **ASR 面板代码:** `KoeApp/Koe/SetupWizard/SPSetupWizardWindowController.m:451-533`
- **豆包协议:** `koe-asr/src/doubao.rs`
- **阿里云协议:** `koe-asr/src/aliyun.rs`
