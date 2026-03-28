# Qwen Provider 测试覆盖 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 Qwen ASR Provider 的核心改动补全测试：段累积逻辑、Final 统一发射、Interim 累积拼接、尾部语气词去除。

**Architecture:** 所有测试都是纯单元测试，通过 `QwenAsrProvider::parse_server_event()` 模拟服务端事件，不需要 mock WebSocket。测试分两类：(1) `strip_trailing_fillers` 独立函数测试；(2) `parse_server_event` 事件解析测试，重点覆盖段累积和 Final 发射逻辑。同时修复一个已过时的旧测试。

**Tech Stack:** Rust, `#[cfg(test)]` inline tests, `cargo test`

---

### Task 1: 修复过时的 `parses_final_transcript` 测试

**背景：** 该测试断言 `completed` 事件会产生 `Definite` + `Final` 两个事件，但代码已改为只产生 `Definite`（Final 改到 `session.finished` 时统一发射）。

**Files:**
- Modify: `koe-asr/src/qwen.rs:438-457`

- [ ] **Step 1: 修复测试断言**

将 `parses_final_transcript` 测试从断言 `Definite` + `Final` 改为只断言 `Definite`（因为 completed 事件不再发射 Final）：

```rust
#[test]
fn parses_completed_segment_as_definite_only() {
    let mut provider = QwenAsrProvider::new();
    let events = provider
        .parse_server_event(
            r#"{
                "type":"conversation.item.input_audio_transcription.completed",
                "transcript":"你好世界"
            }"#,
        )
        .unwrap();

    // completed 事件现在只发射 Definite，Final 留到 session.finished 统一发射
    assert_eq!(events.len(), 1);
    assert!(matches!(
        events.first(),
        Some(AsrEvent::Definite(text)) if text == "你好世界"
    ));
}
```

- [ ] **Step 2: 运行测试确认通过**

Run: `cargo test --manifest-path koe-asr/Cargo.toml -p koe-asr parses_completed`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add koe-asr/src/qwen.rs
git commit -m "test(qwen): fix outdated parses_final_transcript test for new Definite-only behavior"
```

---

### Task 2: 补充 `strip_trailing_fillers` 函数测试

**背景：** 该函数是新增的，完全无测试覆盖。它是纯函数，最容易测试。

**Files:**
- Modify: `koe-asr/src/qwen.rs` (在 `mod tests` 块中追加)

- [ ] **Step 1: 编写 `strip_trailing_fillers` 的测试**

由于 `strip_trailing_fillers` 是 `qwen.rs` 的私有函数，测试需要放在同一文件的 `mod tests` 块中。追加以下测试：

```rust
#[test]
fn strip_trailing_fillers_removes_single_trailing_filler() {
    assert_eq!(strip_trailing_fillers("你好世界嗯"), "你好世界");
}

#[test]
fn strip_trailing_fillers_removes_multiple_trailing_fillers() {
    assert_eq!(strip_trailing_fillers("你好世界嗯啊呃"), "你好世界");
}

#[test]
fn strip_trailing_fillers_removes_filler_with_trailing_punctuation() {
    assert_eq!(strip_trailing_fillers("你好世界嗯，"), "你好世界");
    assert_eq!(strip_trailing_fillers("你好世界啊。"), "你好世界");
}

#[test]
fn strip_trailing_fillers_does_not_remove_leading_fillers() {
    assert_eq!(strip_trailing_fillers("嗯你好世界"), "嗯你好世界");
}

#[test]
fn strip_trailing_fillers_does_not_remove_mid_sentence_fillers() {
    assert_eq!(strip_trailing_fillers("你好嗯世界"), "你好嗯世界");
}

#[test]
fn strip_trailing_fillers_preserves_normal_text() {
    assert_eq!(strip_trailing_fillers("今天天气不错"), "今天天气不错");
}

#[test]
fn strip_trailing_fillers_all_fillers_returns_empty() {
    assert_eq!(strip_trailing_fillers("嗯啊呃"), "");
}

#[test]
fn strip_trailing_fillers_empty_string() {
    assert_eq!(strip_trailing_fillers(""), "");
}
```

- [ ] **Step 2: 运行测试确认全部通过**

Run: `cargo test --manifest-path koe-asr/Cargo.toml -p koe-asr strip_trailing`
Expected: 8 tests PASS

- [ ] **Step 3: Commit**

```bash
git add koe-asr/src/qwen.rs
git commit -m "test(qwen): add comprehensive tests for strip_trailing_fillers"
```

---

### Task 3: 补充段累积 + Interim 拼接逻辑测试

**背景：** 这是本次改动的核心逻辑：多个 VAD 段的文本累积、Interim 事件拼接已确认文本、`session.finished` 统一发射 Final。

**Files:**
- Modify: `koe-asr/src/qwen.rs` (在 `mod tests` 块中追加)

- [ ] **Step 1: 编写段累积和多段 Interim 拼接测试**

```rust
#[test]
fn interim_event_prepends_accumulated_text() {
    let mut provider = QwenAsrProvider::new();

    // 第一个段完成，累积 "今天天气"
    let events = provider
        .parse_server_event(
            r#"{
                "type":"conversation.item.input_audio_transcription.completed",
                "transcript":"今天天气"
            }"#,
        )
        .unwrap();
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0], AsrEvent::Definite(ref t) if t == "今天天气"));

    // 第二个段的 Interim 应该包含已累积的文本
    let events = provider
        .parse_server_event(
            r#"{
                "type":"conversation.item.input_audio_transcription.text",
                "text":"我们去",
                "stash":"公园吧"
            }"#,
        )
        .unwrap();
    assert_eq!(events.len(), 1);
    assert!(matches!(
        events[0],
        AsrEvent::Interim(ref t) if t == "今天天气我们去公园吧"
    ));
}

#[test]
fn first_interim_has_no_prefix() {
    let mut provider = QwenAsrProvider::new();
    // 还没有任何段完成时，Interim 不应有前缀
    let events = provider
        .parse_server_event(
            r#"{
                "type":"conversation.item.input_audio_transcription.text",
                "text":"你好",
                "stash":"世界"
            }"#,
        )
        .unwrap();
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0], AsrEvent::Interim(ref t) if t == "你好世界"));
}

#[test]
fn session_finished_emits_accumulated_final() {
    let mut provider = QwenAsrProvider::new();

    // 两个段完成
    provider.parse_server_event(
        r#"{"type":"conversation.item.input_audio_transcription.completed","transcript":"第一句"}"#,
    ).unwrap();
    provider.parse_server_event(
        r#"{"type":"conversation.item.input_audio_transcription.completed","transcript":"第二句"}"#,
    ).unwrap();

    // session.finished 应该发射累积的 Final
    let events = provider
        .parse_server_event(r#"{"type":"session.finished"}"#)
        .unwrap();

    assert_eq!(events.len(), 2);
    assert!(matches!(
        events[0],
        AsrEvent::Final(ref t) if t == "第一句第二句"
    ));
    assert!(matches!(events[1], AsrEvent::Closed));
}

#[test]
fn session_finished_without_segments_emits_only_closed() {
    let mut provider = QwenAsrProvider::new();
    let events = provider
        .parse_server_event(r#"{"type":"session.finished"}"#)
        .unwrap();
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0], AsrEvent::Closed));
}
```

- [ ] **Step 2: 运行测试确认全部通过**

Run: `cargo test --manifest-path koe-asr/Cargo.toml -p koe-asr`
Expected: 所有旧测试 + 新测试 PASS

- [ ] **Step 3: Commit**

```bash
git add koe-asr/src/qwen.rs
git commit -m "test(qwen): add tests for segment accumulation and unified Final emission"
```

---

### Task 4: 补充其他事件类型和边界情况的测试

**背景：** `session.created`、`session.updated`、`error`、unknown 事件类型以及空 transcript 的 completed 事件都没有测试。

**Files:**
- Modify: `koe-asr/src/qwen.rs` (在 `mod tests` 块中追加)

- [ ] **Step 1: 编写其他事件类型和边界测试**

```rust
#[test]
fn session_created_emits_no_events() {
    let mut provider = QwenAsrProvider::new();
    let events = provider
        .parse_server_event(r#"{"type":"session.created"}"#)
        .unwrap();
    assert!(events.is_empty());
}

#[test]
fn session_updated_emits_connected() {
    let mut provider = QwenAsrProvider::new();
    let events = provider
        .parse_server_event(r#"{"type":"session.updated"}"#)
        .unwrap();
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0], AsrEvent::Connected));
}

#[test]
fn error_event_emits_error() {
    let mut provider = QwenAsrProvider::new();
    let events = provider
        .parse_server_event(
            r#"{"type":"error","error":{"message":"auth failed"}}"#,
        )
        .unwrap();
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0], AsrEvent::Error(ref msg) if msg == "auth failed"));
}

#[test]
fn unknown_event_type_emits_nothing() {
    let mut provider = QwenAsrProvider::new();
    let events = provider
        .parse_server_event(r#"{"type":"some.future.event"}"#)
        .unwrap();
    assert!(events.is_empty());
}

#[test]
fn completed_with_empty_transcript_emits_nothing() {
    let mut provider = QwenAsrProvider::new();
    let events = provider
        .parse_server_event(
            r#"{"type":"conversation.item.input_audio_transcription.completed","transcript":""}"#,
        )
        .unwrap();
    assert!(events.is_empty());
}

#[test]
fn completed_with_nested_transcript_path() {
    let mut provider = QwenAsrProvider::new();
    // 测试 item.content[0].transcript 备选路径
    let events = provider
        .parse_server_event(
            r#"{
                "type":"conversation.item.input_audio_transcription.completed",
                "item":{"content":[{"transcript":"嵌套路径文本"}]}
            }"#,
        )
        .unwrap();
    assert_eq!(events.len(), 1);
    assert!(matches!(
        events[0],
        AsrEvent::Definite(ref t) if t == "嵌套路径文本"
    ));
}
```

- [ ] **Step 2: 运行全部测试**

Run: `cargo test --manifest-path koe-asr/Cargo.toml -p koe-asr`
Expected: 全部 PASS

- [ ] **Step 3: Commit**

```bash
git add koe-asr/src/qwen.rs
git commit -m "test(qwen): add tests for remaining event types and edge cases"
```

---

### Task 5: 更新 Interim 测试以反映累积拼接行为

**背景：** 旧的 `parses_interim_preview_from_text_and_stash` 测试断言 Interim 为 `"今天天气不错"`，现在行为已改为拼接已累积文本。需要更新测试名称和断言。

**Files:**
- Modify: `koe-asr/src/qwen.rs:419-436`

- [ ] **Step 1: 更新 Interim 测试名称和断言**

将 `parses_interim_preview_from_text_and_stash` 重命名并更新，使其在已有累积文本时验证拼接行为：

```rust
#[test]
fn interim_prepends_accumulated_text_to_preview() {
    let mut provider = QwenAsrProvider::new();

    // 先完成一个段，累积 "前面的话"
    provider.parse_server_event(
        r#"{"type":"conversation.item.input_audio_transcription.completed","transcript":"前面的话"}"#,
    ).unwrap();

    // Interim 应该包含已累积文本 + 当前段预览
    let events = provider
        .parse_server_event(
            r#"{
                "type":"conversation.item.input_audio_transcription.text",
                "text":"今天",
                "stash":"天气不错"
            }"#,
        )
        .unwrap();

    assert_eq!(events.len(), 1);
    assert!(matches!(
        events.first(),
        Some(AsrEvent::Interim(text)) if text == "前面的话今天天气不错"
    ));
}
```

- [ ] **Step 2: 运行全部测试确认通过**

Run: `cargo test --manifest-path koe-asr/Cargo.toml -p koe-asr`
Expected: 全部 PASS

- [ ] **Step 3: Commit**

```bash
git add koe-asr/src/qwen.rs
git commit -m "test(qwen): update interim test to reflect accumulated text behavior"
```

---

## Summary

| Task | 新增测试数 | 修复测试数 | 覆盖内容 |
|------|-----------|-----------|---------|
| Task 1 | 0 | 1 | `completed` 事件只发 Definite |
| Task 2 | 8 | 0 | `strip_trailing_fillers` 全场景 |
| Task 3 | 4 | 0 | 段累积 + Interim 拼接 + Final 统一发射 |
| Task 4 | 6 | 0 | 其他事件类型 + 边界情况 |
| Task 5 | 0 | 1 | Interim 累积拼接行为更新 |
| **合计** | **18** | **2** | |
