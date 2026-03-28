---
date: 2026-03-27
topic: asr-test-connection
---

# ASR 配置测试连接按钮

## Problem Frame
用户在配置 ASR（语音识别）服务时，需要填写豆包或阿里云的 API Key。目前用户只能在实际使用时才知道配置是否正确，如果配置错误会导致语音识别失败，体验不佳。需要在配置界面添加"测试连接"按钮，让用户在保存配置前就能验证 API Key 是否有效。

## Requirements

- R1. 在 ASR 配置面板添加"测试连接"按钮，位置与 LLM 配置面板的测试按钮保持一致
- R2. 测试按钮应根据当前选中的 ASR 提供商（豆包/阿里云）测试对应的配置
- R3. 点击测试按钮后，应尝试建立与 ASR 服务的真实 WebSocket 连接
- R4. 测试结果应显示在按钮下方：
  - 成功：绿色文字显示"连接成功"
  - 失败：红色文字显示具体错误信息（如认证失败、网络错误等）
- R5. 测试过程中按钮应禁用，显示"测试中..."
- R6. 测试应在 10 秒内超时，避免长时间等待

## Success Criteria
- 用户可以在保存配置前验证 ASR API Key 是否有效
- 无效的 API Key 能给出明确的错误提示
- 测试过程不影响现有功能

## Scope Boundaries
- 仅测试连接建立和认证，不发送实际音频数据
- 不支持测试语音识别准确性
- 如果用户未填写 API Key，直接提示"请先填写 API Key"

## Key Decisions
- 使用真实连接测试（方案B）：虽然实现较复杂，但能真正验证 API Key 的有效性，比简单的格式验证更有价值
- 测试超时设为 10 秒：豆包和阿里云的连接通常在 3-5 秒内完成，10 秒足够覆盖网络波动情况

## Dependencies / Assumptions
- 需要 Rust FFI 暴露新的测试接口
- 豆包和阿里云都支持仅建立连接而不发送音频的测试方式

## Outstanding Questions

### Resolve Before Planning
- [Affects R3][Technical] 豆包 ASR 是否支持仅建立连接而不发送音频数据的测试方式？需要验证建立 WebSocket 连接后是否能立即收到服务器响应。

### Deferred to Planning
- [Affects R2][Needs research] 具体的 FFI 接口设计：同步阻塞调用还是异步回调？
- [Affects R4][Needs research] 错误信息的本地化：是否需要中英文错误提示？

## Next Steps
→ `/ce:plan` for structured implementation planning
