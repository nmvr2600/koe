# Koe Zen (声·禅)

A lightweight, zen-like macOS voice input tool. Press a hotkey, speak, and the corrected text is pasted into whatever app you're using.

> **Koe Zen** is a focused fork of [Koe](https://github.com/missuo/koe) that prioritizes simplicity and efficiency. No local ASR bloat — just pure cloud-based voice input that stays out of your way.

## The Name

**Koe** (声, pronounced "ko-eh") is the Japanese word for *voice*. Written as こえ in hiragana, it's one of the most fundamental words in the language — simple, clear, and direct.

**Zen** (禅) represents the mindset: no clutter, no bloat, no distractions. Just the essentials — your voice, cleanly transcribed, with minimal footprint.

Koe Zen strips away the complexity to focus on what matters: fast, reliable voice input that respects your system resources.

## Why Koe Zen?

I tried nearly every voice input app on the market. They were either paid, ugly, or inconvenient — bloated UIs, clunky dictionary management, and too many clicks to do simple things.

Koe Zen takes a different approach from the original Koe:

- **No local ASR bloat.** Unlike upstream, Koe Zen uses only cloud-based ASR (Doubao, Qwen). No 80MB+ binaries, no 2GB memory spikes. Just ~15MB app size and ~20MB RAM.
- **ASR Provider switching.** Quickly switch between cloud providers from the menu bar.
- **Everything else you love.** All the simplicity, speed, and reliability of the original.

- **Minimal runtime UI.** Koe Zen stays out of the way with a menu bar item, a small floating status pill with native frosted-glass vibrancy during active sessions, and an optional built-in settings window when you actually need to configure it.
- **All configuration lives in plain text files** under `~/.koe/`. You can edit them with any text editor, vim, a script, or the built-in settings UI.
- **Dictionary is a plain `.txt` file.** No need to open an app and add words one by one through a GUI. Just edit `~/.koe/dictionary.txt` — one term per line. You can even use Claude Code or other AI tools to bulk-generate domain-specific terms.
- **Changes take effect immediately.** Edit any config file and the new settings are used automatically. ASR, LLM, dictionary, and prompt changes apply on the next hotkey press. Hotkey changes are detected within a few seconds. No restart, no reload button.
- **Tiny footprint.** Even after installation, Koe Zen stays **under 15 MB**, and its memory usage is typically **around 20 MB**. It launches fast, wastes almost no disk space, and stays out of your way.
- **Built with native macOS technologies.** Objective-C handles hotkeys, audio capture, clipboard access, permissions, and paste automation directly through Apple's own APIs.
- **Rust does the heavy lifting.** The performance-critical core runs in Rust, which gives Koe Zen low overhead, fast execution, and strong memory safety guarantees.
- **No Chromium tax.** Many comparable Electron-based apps ship at **200+ MB** and carry the overhead of an embedded Chromium runtime. Koe Zen avoids that entire stack, which helps keep memory usage low and the app feeling lightweight.

## How It Works

1. Press and hold the trigger key (default: **Fn**, configurable) — Koe Zen starts listening
2. Audio streams in real-time to a cloud ASR service (Doubao/豆包 or Qwen/通义)
3. A floating status pill shows real-time interim recognition text as you speak
4. The ASR transcript is corrected by an LLM (any OpenAI-compatible API) — fixing capitalization, punctuation, spacing, and terminology
5. The corrected text is automatically pasted into the active input field

ASR provider support:

- **Cloud**: **Doubao (豆包)** and **Qwen (通义)** streaming ASR
- **LLM**: any **OpenAI-compatible API** for text correction

## Installation

### Build from Source

#### Prerequisites

- macOS 13.0+
- Apple Silicon or Intel Mac
- Rust toolchain (`rustup`)
- Xcode with command line tools
- [xcodegen](https://github.com/yonaskolb/XcodeGen) (`brew install xcodegen`)

#### Build

```bash
git clone https://github.com/nmvr2600/koe.git
cd koe

# Generate Xcode project
cd KoeApp && xcodegen && cd ..

# Build Apple Silicon
make build

# Build Intel
make build-x86_64
```

#### Run

```bash
make run
```

Or open the built app directly:

```bash
open ~/Library/Developer/Xcode/DerivedData/Koe-*/Build/Products/Release/Koe.app
```

### Permissions

Koe Zen requires **three macOS permissions** to function. You'll be prompted to grant them on first launch. All three are mandatory — without any one of them, Koe Zen cannot complete its core workflow.

| Permission | Why it's needed | What happens without it |
|---|---|---|
| **Microphone** | Captures audio from your mic and streams it to the ASR service for speech recognition. | Koe Zen cannot hear you at all. Recording will not start. |
| **Accessibility** | Simulates a `Cmd+V` keystroke to paste the corrected text into the active input field of any app. | Koe Zen will still copy the text to your clipboard, but cannot auto-paste. You'll need to paste manually. |
| **Input Monitoring** | Listens for the trigger key (default: **Fn**, configurable) globally so Koe Zen can detect when you press/release it, regardless of which app is in the foreground. | Koe Zen cannot detect the hotkey. You won't be able to trigger recording. |

To grant permissions: **System Settings → Privacy & Security** → enable Koe Zen under each of the three categories above.

## Configuration

All config files live in `~/.koe/` and are auto-generated on first launch. You can edit them directly, or use the built-in settings window from the menu bar.

```
~/.koe/
├── config.yaml          # Main configuration
├── dictionary.txt       # User dictionary (hotwords + LLM correction)
├── history.db           # Usage statistics (SQLite, auto-created)
├── system_prompt.txt    # LLM system prompt (customizable)
└── user_prompt.txt      # LLM user prompt template (customizable)
```

### config.yaml

Below is the full configuration with explanations for every field.

#### ASR (Speech Recognition)

Koe Zen uses a provider-based ASR config layout with cloud providers only.

```yaml
asr:
  # ASR provider: "doubao" or "qwen"
  provider: "doubao"

  doubao:
    # WebSocket endpoint. Default uses ASR 2.0 optimized bidirectional streaming.
    url: "wss://openspeech.bytedance.com/api/v3/sauc/bigmodel_async"

    # Volcengine credentials — get these from the 火山引擎 console.
    # Go to: https://console.volcengine.com/speech/app → create an app → copy App ID and Access Token.
    app_key: ""          # X-Api-App-Key (火山引擎 App ID)
    access_key: ""       # X-Api-Access-Key (火山引擎 Access Token)

    # Resource ID for billing. Default is the standard duration-based billing plan.
    resource_id: "volc.seedasr.sauc.duration"

    # Connection timeout in milliseconds. Increase if you have slow network.
    connect_timeout_ms: 3000

    # How long to wait for the final ASR result after you stop speaking (ms).
    final_wait_timeout_ms: 5000

    # Disfluency removal (语义顺滑). Removes spoken repetitions and filler words like 嗯, 那个.
    enable_ddc: true

    # Inverse text normalization (文本规范化). Converts spoken numbers, dates, etc.
    enable_itn: true

    # Automatic punctuation.
    enable_punc: true

    # Two-pass recognition (二遍识别). First pass gives fast streaming results,
    # second pass re-recognizes with higher accuracy.
    enable_nonstream: true

  # Qwen (Aliyun DashScope) Realtime ASR
  qwen:
    url: "wss://dashscope.aliyuncs.com/api-ws/v1/realtime"
    api_key: ""
    model: "qwen3-asr-flash-realtime"
    language: "zh"
    connect_timeout_ms: 3000
    final_wait_timeout_ms: 5000
```

#### LLM (Text Correction)

After ASR, the transcript is sent to an LLM for correction (capitalization, spacing, terminology, filler word removal).

```yaml
llm:
  # Set to false to skip LLM correction and paste raw ASR output directly.
  enabled: true

  # OpenAI-compatible API endpoint.
  base_url: "https://api.openai.com/v1"

  # API key. Supports environment variable substitution with ${VAR_NAME} syntax.
  api_key: ""

  # Model name. Use a fast, cheap model — latency matters here.
  model: "gpt-5.4-nano"

  # LLM sampling parameters. temperature: 0 = deterministic.
  temperature: 0
  top_p: 1

  # LLM request timeout in milliseconds.
  timeout_ms: 8000

  # Max tokens in LLM response.
  max_output_tokens: 1024

  # Token limit field sent to the OpenAI-compatible API.
  max_token_parameter: "max_completion_tokens"

  # How many dictionary entries to include in the LLM prompt.
  dictionary_max_candidates: 0

  # Paths to prompt files, relative to ~/.koe/.
  system_prompt_path: "system_prompt.txt"
  user_prompt_path: "user_prompt.txt"
```

#### Hotkey

```yaml
hotkey:
  # Trigger key for voice input.
  # Options: fn | left_option | right_option | left_command | right_command | left_control | right_control
  # Or a raw keycode number (e.g. 122 for F1) for non-modifier keys.
  trigger_key: "fn"
  # Cancel key for aborting the current session.
  cancel_key: "left_option"
```

### Dictionary

The dictionary serves two purposes:

1. **ASR hotwords** — sent to the speech recognition engine to improve accuracy for specific terms
2. **LLM correction** — included in the prompt so the LLM prefers these spellings and terms

Edit `~/.koe/dictionary.txt`:

```
# One term per line. Lines starting with # are comments.
Cloudflare
PostgreSQL
Kubernetes
GitHub Actions
VS Code
```

## Usage Statistics

Koe Zen automatically tracks your voice input usage in a local SQLite database at `~/.koe/history.db`. You can view a summary directly in the menu bar dropdown.

## Architecture

Koe Zen is built as a native macOS app with two layers:

- **Objective-C shell** — handles macOS integration: hotkey detection, audio capture, clipboard management, paste simulation, menu bar UI, and usage statistics (SQLite)
- **Rust core library** — handles ASR (cloud WebSocket streaming), LLM API calls, config management, transcript aggregation, and session orchestration

The two layers communicate via C FFI (Foreign Function Interface). The Rust core is compiled as a static library (`libkoe_core.a`) and linked into the Xcode project.

## License

MIT
