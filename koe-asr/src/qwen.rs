use crate::config::AsrConfig;
use crate::error::{AsrError, Result};
use crate::event::AsrEvent;
use crate::provider::AsrProvider;
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use std::collections::VecDeque;
use tokio::time::{timeout, Duration};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use uuid::Uuid;

type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

const DASHSCOPE_WS_URL: &str =
    "wss://dashscope.aliyuncs.com/api-ws/v1/realtime?model=qwen3-asr-flash-realtime";
const SESSION_EVENT_TIMEOUT: Duration = Duration::from_secs(5);

// VAD (语音活动检测) 参数配置
// threshold: 语音检测阈值，推荐 0.0，API 默认 0.2
// 过低容易把呼吸声识别为"嗯啊"语气词，过高会降低灵敏度
const VAD_THRESHOLD: f32 = 0.2;
// silence_duration_ms: 静音持续时间，超过此值视为语音结束
// 较低的值(300ms)响应快但容易在自然停顿处断句，较高的值(1200ms)能容忍长句中的停顿
const VAD_SILENCE_DURATION_MS: u32 = 800;
// prefix_padding_ms: 语音开始前保留的音频时长，用于捕捉语音起始部分
const VAD_PREFIX_PADDING_MS: u32 = 100;

/// Qwen DashScope 实时语音识别 Provider (Qwen-ASR-Realtime)
///
/// 协议参考Qwen官方 WebSocket Realtime API：
/// 1. 连接建立后等待 `session.created`
/// 2. 发送 `session.update`
/// 3. 使用 `input_audio_buffer.append` 追加 Base64 音频
/// 4. 音频结束后发送 `session.finish`
pub struct QwenAsrProvider {
    ws: Option<WsStream>,
    input_finished: bool,
    pending_events: VecDeque<AsrEvent>,
    // 累积所有 VAD 段的最终文本，等 session.finished 时统一发射 Final
    accumulated_text: String,
}

impl QwenAsrProvider {
    pub fn new() -> Self {
        Self {
            ws: None,
            input_finished: false,
            pending_events: VecDeque::new(),
            accumulated_text: String::new(),
        }
    }

    fn build_session_update(config: &AsrConfig) -> ClientEvent {
        // Qwen ASR uses "zh" (Chinese) as default language
        // Language is Qwen-specific, so it's hardcoded here rather than in shared AsrConfig
        const DEFAULT_LANGUAGE: &str = "zh";
        ClientEvent {
            event_id: format!("event_{}", Uuid::new_v4()),
            event_type: "session.update".to_string(),
            audio: None,
            session: Some(serde_json::json!({
                "modalities": ["text"],
                "input_audio_format": "pcm",
                "sample_rate": config.sample_rate_hz,
                "input_audio_transcription": {
                    "model": "qwen3-asr-flash-realtime",
                    "language": DEFAULT_LANGUAGE,
                },
                "turn_detection": {
                    "type": "server_vad",
                    "threshold": VAD_THRESHOLD,
                    "silence_duration_ms": VAD_SILENCE_DURATION_MS,
                    "prefix_padding_ms": VAD_PREFIX_PADDING_MS,
                }
            })),
        }
    }

    fn build_audio_append(audio_data: &[u8]) -> ClientEvent {
        use base64::{Engine, engine::general_purpose::STANDARD};
        ClientEvent {
            event_id: format!("event_{}", Uuid::new_v4()),
            event_type: "input_audio_buffer.append".to_string(),
            audio: Some(STANDARD.encode(audio_data)),
            session: None,
        }
    }

    fn build_session_finish() -> ClientEvent {
        ClientEvent {
            event_id: format!("event_{}", Uuid::new_v4()),
            event_type: "session.finish".to_string(),
            audio: None,
            session: None,
        }
    }

    fn parse_server_event(&mut self, text: &str) -> Result<Vec<AsrEvent>> {
        log::debug!("[Qwen ASR] Received: {}", text);

        let raw_json: serde_json::Value = serde_json::from_str(text)
            .map_err(|e| AsrError::Protocol(format!("parse server event: {e}")))?;

        let event_type = raw_json
            .get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("unknown");

        let mut events = Vec::new();

        match event_type {
            "session.created" => {
                log::info!("[Qwen ASR] Session created");
            }
            "session.updated" => {
                log::info!("[Qwen ASR] Session updated");
                events.push(AsrEvent::Connected);
            }
            "input_audio_buffer.speech_started" => {
                log::debug!("[Qwen ASR] Speech started");
            }
            "input_audio_buffer.speech_stopped" => {
                log::debug!("[Qwen ASR] Speech stopped");
            }
            "input_audio_buffer.committed" => {
                log::debug!("[Qwen ASR] Audio buffer committed");
            }
            "conversation.item.created" => {
                log::debug!("[Qwen ASR] Conversation item created");
            }
            "conversation.item.input_audio_transcription.text" => {
                let text = raw_json.get("text").and_then(|v| v.as_str()).unwrap_or("");
                let stash = raw_json.get("stash").and_then(|v| v.as_str()).unwrap_or("");
                let preview = format!("{text}{stash}");
                if !preview.is_empty() {
                    // 把之前已确认的段文本拼到前面，让预览显示完整内容
                    let full_preview = if self.accumulated_text.is_empty() {
                        preview
                    } else {
                        format!("{}{}", self.accumulated_text, preview)
                    };
                    events.push(AsrEvent::Interim(full_preview));
                }
            }
            "conversation.item.input_audio_transcription.completed" => {
                let transcript = raw_json
                    .get("transcript")
                    .and_then(|v| v.as_str())
                    .or_else(|| {
                        raw_json
                            .get("item")
                            .and_then(|i| i.get("content"))
                            .and_then(|c| c.as_array())
                            .and_then(|arr| arr.first())
                            .and_then(|content| content.get("transcript"))
                            .and_then(|v| v.as_str())
                    })
                    .unwrap_or("");

                if !transcript.is_empty() {
                    // 累积到总文本
                    if !self.accumulated_text.is_empty() {
                        self.accumulated_text.push_str(&transcript);
                    } else {
                        self.accumulated_text = transcript.to_string();
                    }
                    log::info!("[Qwen ASR] Segment final: {} (accumulated: {} chars)", transcript, self.accumulated_text.len());
                    // 发射包含所有已确认段的 Definite，让 best_text() 返回累积文本
                    events.push(AsrEvent::Definite(self.accumulated_text.clone()));
                }
            }
            "session.finished" => {
                log::info!("[Qwen ASR] Session finished");
                // 统一发射累积的完整文本作为 Final
                if !self.accumulated_text.is_empty() {
                    // 去掉尾部常见的单字语气词（千问容易在句末产生）
                    let cleaned = strip_trailing_fillers(&self.accumulated_text);
                    if cleaned != self.accumulated_text {
                        log::info!("[Qwen ASR] Stripped trailing fillers: '{}' -> '{}'", self.accumulated_text, cleaned);
                    }
                    log::info!("[Qwen ASR] Emitting accumulated final: {} chars", cleaned.len());
                    events.push(AsrEvent::Final(cleaned));
                }
                events.push(AsrEvent::Closed);
            }
            "error" => {
                let error_msg = raw_json
                    .get("error")
                    .and_then(|e| e.get("message"))
                    .and_then(|m| m.as_str())
                    .unwrap_or("Unknown error");
                log::error!("[Qwen ASR] Error: {}", error_msg);
                events.push(AsrEvent::Error(error_msg.to_string()));
            }
            other => {
                log::debug!("[Qwen ASR] Ignoring event type: {}", other);
            }
        }

        Ok(events)
    }

    async fn read_text_event(ws: &mut WsStream, timeout_duration: Duration) -> Result<String> {
        match timeout(timeout_duration, ws.next()).await {
            Ok(Some(Ok(Message::Text(text)))) => Ok(text.to_string()),
            Ok(Some(Ok(Message::Close(frame)))) => Err(AsrError::Connection(format!(
                "connection closed unexpectedly: {:?}",
                frame
            ))),
            Ok(Some(Ok(_))) => Err(AsrError::Connection(
                "expected text message from server".into(),
            )),
            Ok(Some(Err(e))) => Err(AsrError::Connection(format!("WebSocket error: {e}"))),
            Ok(None) => Err(AsrError::Connection("connection closed".into())),
            Err(_) => Err(AsrError::Connection(
                "timeout waiting for server event".into(),
            )),
        }
    }

    async fn send_client_event(&mut self, event: ClientEvent) -> Result<()> {
        let msg_text = serde_json::to_string(&event)
            .map_err(|e| AsrError::Protocol(format!("serialize client event: {e}")))?;

        if let Some(ref mut ws) = self.ws {
            ws.send(Message::Text(msg_text.into()))
                .await
                .map_err(|e| AsrError::Protocol(format!("send client event: {e}")))?;
            Ok(())
        } else {
            Err(AsrError::Connection("not connected".into()))
        }
    }
}

impl Default for QwenAsrProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl AsrProvider for QwenAsrProvider {
    async fn connect(&mut self, config: &AsrConfig) -> Result<()> {
        let api_key = config.access_key.clone();
        if api_key.is_empty() {
            return Err(AsrError::Connection("api_key is required".into()));
        }

        log::info!("Connecting to Qwen ASR: {}", DASHSCOPE_WS_URL);

        let mut request = DASHSCOPE_WS_URL
            .into_client_request()
            .map_err(|e| AsrError::Connection(format!("invalid URL: {e}")))?;

        request.headers_mut().insert(
            "Authorization",
            format!("Bearer {}", api_key)
                .parse()
                .map_err(|_| AsrError::Connection("invalid api_key".into()))?,
        );

        let (ws_stream, response) =
            timeout(Duration::from_millis(config.connect_timeout_ms), async {
                connect_async(request)
                    .await
                    .map_err(|e| AsrError::Connection(e.to_string()))
            })
            .await
            .map_err(|_| AsrError::Connection("connection timed out".into()))??;

        log::info!("[Qwen ASR] WebSocket connected: {}", response.status());
        self.ws = Some(ws_stream);

        if let Some(ref mut ws) = self.ws {
            let created_text = Self::read_text_event(ws, SESSION_EVENT_TIMEOUT).await?;
            let created_json: serde_json::Value = serde_json::from_str(&created_text)
                .map_err(|e| AsrError::Protocol(format!("parse session.created: {e}")))?;
            let created_type = created_json
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            if created_type != "session.created" {
                let error_message = created_json
                    .get("error")
                    .and_then(|e| e.get("message"))
                    .and_then(|m| m.as_str())
                    .unwrap_or("expected session.created event");
                return Err(AsrError::Connection(error_message.to_string()));
            }
        }

        self.send_client_event(Self::build_session_update(config))
            .await?;

        loop {
            let event = self.next_event().await?;
            match event {
                AsrEvent::Connected => break,
                AsrEvent::Error(msg) => return Err(AsrError::Protocol(msg)),
                AsrEvent::Closed => {
                    return Err(AsrError::Connection(
                        "connection closed before session.updated".into(),
                    ))
                }
                AsrEvent::Interim(_) | AsrEvent::Definite(_) | AsrEvent::Final(_) => {
                    log::debug!("[Qwen ASR] Received transcript before session.updated");
                }
            }
        }

        log::info!("Qwen ASR connected and configured");
        Ok(())
    }

    async fn send_audio(&mut self, frame: &[u8]) -> Result<()> {
        if frame.is_empty() {
            return Ok(());
        }

        self.send_client_event(Self::build_audio_append(frame))
            .await
    }

    async fn finish_input(&mut self) -> Result<()> {
        if self.input_finished {
            return Ok(());
        }

        self.input_finished = true;
        self.send_client_event(Self::build_session_finish()).await
    }

    async fn next_event(&mut self) -> Result<AsrEvent> {
        if let Some(event) = self.pending_events.pop_front() {
            return Ok(event);
        }

        if let Some(ref mut ws) = self.ws {
            match ws.next().await {
                Some(Ok(Message::Text(text))) => {
                    let events = self.parse_server_event(&text)?;
                    self.pending_events.extend(events);
                    Ok(self
                        .pending_events
                        .pop_front()
                        .unwrap_or_else(|| AsrEvent::Interim(String::new())))
                }
                Some(Ok(Message::Close(_))) => Ok(AsrEvent::Closed),
                Some(Ok(Message::Binary(data))) => {
                    log::debug!(
                        "[Qwen ASR] Ignoring binary message ({} bytes)",
                        data.len()
                    );
                    Ok(AsrEvent::Interim(String::new()))
                }
                Some(Ok(_)) => Ok(AsrEvent::Interim(String::new())),
                Some(Err(e)) => Err(AsrError::Protocol(e.to_string())),
                None => Ok(AsrEvent::Closed),
            }
        } else {
            Err(AsrError::Connection("not connected".into()))
        }
    }

    async fn close(&mut self) -> Result<()> {
        if let Some(mut ws) = self.ws.take() {
            let _ = ws.close(None).await;
        }
        Ok(())
    }
}

/// 去掉文本尾部的单字语气词（千问 ASR 容易在句末/段末产生）
fn strip_trailing_fillers(text: &str) -> String {
    const FILLERS: &[char] = &['嗯', '啊', '呃', '哦', '呀', '噢', '唔', '额', '呢', '吧'];
    let trimmed = text.trim_end();
    let mut end = trimmed.len();
    // 从尾部开始，连续去掉语气词 + 可选的标点/空格
    while end > 0 {
        let prev = if end > 0 { trimmed[..end].trim_end_matches(|c: char| c == '，' || c == ',' || c == '。' || c == ' ' || c == '、') } else { trimmed };
        if prev.is_empty() {
            break;
        }
        let last_char = prev.chars().last().unwrap();
        if FILLERS.contains(&last_char) {
            end = prev.len() - last_char.len_utf8();
        } else {
            break;
        }
    }
    if end == 0 {
        String::new()
    } else {
        trimmed[..end].to_string()
    }
}

#[derive(Serialize)]
struct ClientEvent {
    #[serde(rename = "event_id")]
    event_id: String,
    #[serde(rename = "type")]
    event_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    audio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    session: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::QwenAsrProvider;
    use crate::event::AsrEvent;

    #[test]
    fn parses_interim_preview_from_text_and_stash() {
        let mut provider = QwenAsrProvider::new();
        let events = provider
            .parse_server_event(
                r#"{
                    "type":"conversation.item.input_audio_transcription.text",
                    "text":"今天",
                    "stash":"天气不错"
                }"#,
            )
            .unwrap();

        assert!(matches!(
            events.first(),
            Some(AsrEvent::Interim(text)) if text == "今天天气不错"
        ));
    }

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
}
