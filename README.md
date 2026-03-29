# Koe Zen (声·禅)

> **Languages**: [中文](README.md) | [English](README.en.md)

一款轻量、禅意的 macOS 语音输入工具。按下热键，说话，修正后的文字自动粘贴到当前输入框。

---

## 关于本项目

**Koe Zen** 是 [Koe](https://github.com/missuo/koe) 的一个分支。

原版的 Koe 是一款优秀的 macOS 语音输入工具，我们在此基础上进行了调整。现在的语音输入软件市场已经相当拥挤——大厂出品、开源方案、各种 DIY 工具层出不穷。我们不想再造一个和它们一模一样的轮子，而是希望保留 Koe 最核心、最纯粹的那部分体验。

**Koe Zen 的定位很简单**：保持精简。

上游版本加入了本地 ASR（MLX、sherpa-onnx）支持，虽然功能更强大，但代价是应用体积从 15MB 膨胀到 86MB，内存占用从 20MB 飙升到近 2GB。对于一款常驻后台的语音输入工具来说，这样的资源消耗违背了"小而美"的初衷。

所以我们做了减法：
- ❌ 移除本地 ASR 支持（MLX、sherpa-onnx）
- ✅ 保留云端 ASR（Doubao、Qwen）—— 快速、准确、零本地资源占用
- ✅ 保留菜单栏 Provider 切换等实用功能
- ✅ 保持 ~15MB 体积、~20MB 内存占用

如果你需要本地语音识别（离线使用、隐私敏感场景），建议直接使用上游原版 [Koe](https://github.com/missuo/koe)。如果你只是想要一个**轻量、快速、不打扰**的云端语音输入工具，Koe Zen 可能更适合你。

---

## 名字的含义

**Koe**（声，发音 "ko-eh"）是日语中"声音"的意思。写成平假名是 こえ，是日语中最基础的词汇之一 —— 简单、清晰、直接。

**Zen**（禅）代表一种心态：没有杂乱、没有臃肿、没有干扰。只有 essentials —— 你的声音，被清晰地转录，以最小的资源占用。

Koe Zen 剥离了复杂性，专注于重要的事情：快速、可靠的语音输入，尊重你的系统资源。

## 为什么选择 Koe Zen？

我试用了市面上几乎所有的语音输入应用。它们要么收费、要么难看、要么不方便 —— 臃肿的界面、笨拙的字典管理、做简单的事情需要太多点击。

Koe Zen 与原版的 Koe 采取了不同的方法：

- **没有本地 ASR 的臃肿。** 与上游不同，Koe Zen 只使用云端 ASR（Doubao、Qwen）。没有 80MB+ 的二进制文件，没有 2GB 的内存飙升。只有约 15MB 的应用大小和约 20MB 的内存占用。
- **ASR Provider 切换。** 从菜单栏快速切换云端提供商。
- **你喜爱的其他一切。** 原版的简洁、速度和可靠性。

- **最小的运行时界面。** Koe Zen 以菜单栏图标常驻，活跃会话期间显示小型浮动状态药丸，需要配置时才显示内置设置窗口。
- **所有配置都存储在纯文本文件中**，位于 `~/.koe/`。你可以用任何文本编辑器、vim、脚本或内置设置 UI 来编辑它们。
- **字典是普通的 `.txt` 文件。** 不需要打开应用一个一个添加。直接编辑 `~/.koe/dictionary.txt` —— 每行一个词。你甚至可以用 Claude Code 或其他 AI 工具批量生成专业术语。
- **更改立即生效。** 编辑任何配置文件，新设置自动生效。ASR、LLM、字典和提示词更改在下一次热键按下时应用。热键更改在几秒钟内被检测。无需重启，无需重新加载按钮。
- **极小的资源占用。** 安装后，Koe Zen 保持在 **15 MB 以下**，内存使用通常在 **20 MB 左右**。启动快，几乎不浪费磁盘空间，不打扰你的工作。
- **使用原生 macOS 技术构建。** Objective-C 直接通过 Apple 自己的 API 处理热键、音频捕获、剪贴板、权限和粘贴自动化。
- **Rust 处理繁重的任务。** 性能关键的核心使用 Rust，这给了 Koe Zen 低开销、快速执行和强大的内存安全保证。
- **没有 Chromium 负担。** 许多类似的 Electron 应用打包后 **200+ MB**，带有嵌入式 Chromium 运行时的开销。Koe Zen 完全避免这种架构，有助于保持低内存使用，让应用感觉轻量。

## 工作原理

1. 按住触发键（默认：**Fn**，可配置）— Koe Zen 开始监听
2. 音频实时流式传输到云端 ASR 服务（Doubao/豆包 或 Qwen/通义）
3. 浮动状态药丸实时显示中间识别结果
4. LLM（任何 OpenAI 兼容 API）修正 ASR 转录 —— 修复大小写、标点、空格和术语
5. 修正后的文字自动粘贴到当前输入框

ASR 提供商支持：

- **云端**：**Doubao（豆包）** 和 **Qwen（通义）** 流式 ASR
- **LLM**：任何 **OpenAI 兼容 API** 用于文本修正

## 安装

### 从源码构建

#### 前置要求

- macOS 13.0+
- Apple Silicon 或 Intel Mac
- Rust 工具链 (`rustup`)
- Xcode 及命令行工具
- [xcodegen](https://github.com/yonaskolb/XcodeGen) (`brew install xcodegen`)

#### 构建

```bash
git clone https://github.com/nmvr2600/koe.git
cd koe

# 生成 Xcode 项目
cd KoeApp && xcodegen && cd ..

# 构建 Apple Silicon 版本
make build

# 构建 Intel 版本
make build-x86_64
```

#### 运行

```bash
make run
```

或直接打开构建好的应用：

```bash
open ~/Library/Developer/Xcode/DerivedData/Koe-*/Build/Products/Release/Koe.app
```

### 权限

Koe Zen 需要 **三个 macOS 权限** 才能工作。首次启动时会提示你授予这些权限。三者缺一不可 —— 缺少任何一个，Koe Zen 都无法完成其核心工作流。

| 权限 | 为什么需要 | 没有它会怎样 |
|---|---|---|
| **麦克风** | 从麦克风捕获音频并流式传输到 ASR 服务进行语音识别。 | Koe Zen 完全听不到你。录音不会开始。 |
| **辅助功能** | 模拟 `Cmd+V` 按键，将修正后的文字粘贴到任何应用的输入框。 | Koe Zen 仍会复制文字到剪贴板，但无法自动粘贴。你需要手动粘贴。 |
| **输入监控** | 全局监听触发键（默认：**Fn**，可配置），这样 Koe Zen 可以在你按下/松开时检测，无论前台是什么应用。 | Koe Zen 无法检测热键。你无法触发录音。 |

要授予权限：**系统设置 → 隐私与安全性** → 在上述三个类别下启用 Koe Zen。

## 配置

所有配置文件都位于 `~/.koe/`，首次启动时自动生成。你可以直接编辑它们，或使用菜单栏的内置设置窗口。

```
~/.koe/
├── config.yaml          # 主配置
├── dictionary.txt       # 用户字典（热词 + LLM 修正）
├── history.db           # 使用统计（SQLite，自动创建）
├── system_prompt.txt    # LLM 系统提示词（可自定义）
└── user_prompt.txt      # LLM 用户提示词模板（可自定义）
```

### config.yaml

以下是完整配置及每个字段的说明。

#### ASR（语音识别）

Koe Zen 使用基于提供商的 ASR 配置布局，仅支持云端提供商。

```yaml
asr:
  # ASR 提供商："doubao" 或 "qwen"
  provider: "doubao"

  doubao:
    # WebSocket 端点。默认使用 ASR 2.0 优化版双向流式。
    url: "wss://openspeech.bytedance.com/api/v3/sauc/bigmodel_async"

    # 火山引擎凭证 —— 从 火山引擎 控制台获取。
    # 访问：https://console.volcengine.com/speech/app → 创建应用 → 复制 App ID 和 Access Token。
    app_key: ""          # X-Api-App-Key（火山引擎 App ID）
    access_key: ""       # X-Api-Access-Key（火山引擎 Access Token）

    # 计费资源 ID。默认是按使用时长计费的标准套餐。
    resource_id: "volc.seedasr.sauc.duration"

    # 连接超时（毫秒）。如果网络较慢，请增加。
    connect_timeout_ms: 3000

    # 停止说话后等待最终 ASR 结果的时间（毫秒）。
    final_wait_timeout_ms: 5000

    # 语义顺滑。去除口语重复和语气词如 嗯、那个。
    enable_ddc: true

    # 文本规范化。将口语数字、日期等转换。
    enable_itn: true

    # 自动标点。
    enable_punc: true

    # 二遍识别。首轮给出快速流式结果，
    # 第二轮以更高准确率重新识别。
    enable_nonstream: true

  # 通义千问（阿里云灵积）实时 ASR
  qwen:
    url: "wss://dashscope.aliyuncs.com/api-ws/v1/realtime"
    api_key: ""
    model: "qwen3-asr-flash-realtime"
    language: "zh"
    connect_timeout_ms: 3000
    final_wait_timeout_ms: 5000
```

#### LLM（文本修正）

ASR 后，转录文本被发送到 LLM 进行修正（大小写、空格、术语、语气词去除）。

```yaml
llm:
  # 设为 false 跳过 LLM 修正，直接粘贴原始 ASR 输出。
  enabled: true

  # OpenAI 兼容 API 端点。
  base_url: "https://api.openai.com/v1"

  # API 密钥。支持 ${VAR_NAME} 语法的环境变量替换。
  api_key: ""

  # 模型名称。使用快速、便宜的模型 —— 延迟很重要。
  model: "gpt-5.4-nano"

  # LLM 采样参数。temperature: 0 = 确定性。
  temperature: 0
  top_p: 1

  # LLM 请求超时（毫秒）。
  timeout_ms: 8000

  # LLM 响应中的最大 token 数。
  max_output_tokens: 1024

  # 发送给 OpenAI 兼容 API 的 token 参数字段。
  max_token_parameter: "max_completion_tokens"

  # 在 LLM 提示词中包含多少字典条目。
  dictionary_max_candidates: 0

  # 提示词文件路径，相对于 ~/.koe/。
  system_prompt_path: "system_prompt.txt"
  user_prompt_path: "user_prompt.txt"
```

#### 热键

```yaml
hotkey:
  # 语音输入触发键。
  # 选项：fn | left_option | right_option | left_command | right_command | left_control | right_control
  # 或修饰键的原始 keycode 数字（如 F1 是 122）。
  trigger_key: "fn"
  # 取消当前会话的按键。
  cancel_key: "left_option"
```

### 字典

字典有两个作用：

1. **ASR 热词** —— 发送给语音识别引擎，提高特定术语的准确率
2. **LLM 修正** —— 包含在提示词中，让 LLM 优先使用这些拼写和术语

编辑 `~/.koe/dictionary.txt`：

```
# 每行一个词。以 # 开头的行是注释。
Cloudflare
PostgreSQL
Kubernetes
GitHub Actions
VS Code
```

## 使用统计

Koe Zen 自动追踪你的语音输入使用情况，存储在本地 SQLite 数据库 `~/.koe/history.db`。你可以直接在菜单栏下拉菜单中查看汇总信息。

## 架构

Koe Zen 是一个原生 macOS 应用，由两层构建：

- **Objective-C 外壳** —— 处理 macOS 集成：热键检测、音频捕获、剪贴板管理、粘贴模拟、菜单栏 UI 和使用统计（SQLite）
- **Rust 核心库** —— 处理 ASR（云端 WebSocket 流式）、LLM API 调用、配置管理、转录聚合和会话编排

两层通过 C FFI（外部函数接口）通信。Rust 核心被编译为静态库（`libkoe_core.a`）并链接到 Xcode 项目中。

## 许可证

MIT
