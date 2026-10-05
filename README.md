# 偶译析 · 翻译解析助手（TransHelper）

> 输入任意语言的文本，获得**中文翻译 + 词汇解析 + 语法分析 + 例句**的一站式深度解读。
>
> 基于 Tauri 2 + Vue 3 构建的轻量级桌面应用，直连大模型 API，无账号、无历史记录、不留云端痕迹。

---

## ✨ 功能特性

- **翻译 + 深度解析**：内置结构化系统提示词，输出固定为「中文翻译 / 词汇解析 / 语法与结构 / 例句」四个板块
- **双协议支持**
  - OpenAI 兼容协议：DeepSeek、通义千问、Kimi、智谱 GLM 等
  - Anthropic 协议：Claude 系列
- **流式输出**：SSE 逐字渲染，实时看到生成过程
- **自动代理**：跟随标准代理环境变量；Linux KDE 下也读取系统手动 HTTP(S) 代理设置
- **单轮追问**：针对翻译结果继续提问，自动携带原文与最近一轮结果作为上下文
- **Markdown 渲染**：回答以 Markdown 排版展示，并经 [DOMPurify](https://github.com/cure53/DOMPurify) 消毒防 XSS
- **思考深度调节**：OpenAI 协议发送 `reasoning_effort`；Anthropic 协议自动开启 extended thinking
- **连接测试**：设置页一键验证 API 配置是否可用
- **轻量隐私友好**：每次只发送当前文本，不积累对话历史；所有数据仅存于本地
- **快捷操作**：`Ctrl + Enter` 快捷发起翻译，支持一键「丢弃」清空会话

## 🛠 技术栈

| 层 | 技术 |
| --- | --- |
| 桌面框架 | [Tauri 2](https://tauri.app)（Rust 后端） |
| 前端 | Vue 3 `<script setup>` + TypeScript + Vite 6 |
| HTTP / 流式 | reqwest（rustls-tls）+ SSE 手动解析，通过 Tauri 事件推送到前端 |
| 渲染 | marked + DOMPurify |

## 📦 环境要求

- **Node.js** ≥ 18（含 npm）
- **Rust** stable 工具链（Windows 下需 MSVC 工具链）
- **Windows 10/11**（内置 WebView2；当前构建脚本面向 Windows NSIS）
- **Linux**：支持 AppImage / deb；运行依赖 GTK 3 与 WebKitGTK 4.1，源码构建还需对应开发包（Fedora：`webkit2gtk4.1-devel`、`gtk3-devel`）

## 🚀 快速开始

```bash
# 1. 安装前端依赖
npm install

# 2. 以桌面应用模式启动开发环境（前端热更新）
npm run tauri dev
```

首次启动后请点击右上角 ⚙ 进入**设置页**，填写 API 配置并保存。

## 🔨 构建 Release

### 方式一：一键脚本（Windows PowerShell）

```powershell
# NSIS 安装包（默认）
.\build-release.ps1

# 便携版单文件 exe
.\build-release.ps1 -Portable

# 可选参数：-SkipInstall 跳过 npm install；-Open 构建完成后打开产物目录
```

### 方式二：npm 脚本

```bash
npm run build:release            # NSIS 安装包
npm run build:release:portable   # 便携版（不打包安装器）
```

产物位置：

| 类型 | 路径 |
| --- | --- |
| 便携版 exe | `src-tauri/target/release/transhelper.exe` |
| 安装包 | `src-tauri/target/release/bundle/nsis/*.exe` |

其他常用命令：

```bash
npm run dev        # 仅启动前端（浏览器调试）
npm run build      # vue-tsc 类型检查 + Vite 前端构建
npm run preview    # 预览前端构建产物
```

## 📖 使用指南

1. **配置 API**（首次必做）：进入设置页 → 选择协议 → 填写 Base URL / API Key / 模型名 → 点击「测试连接」→ 保存
2. **翻译**：在主界面粘贴任意语言文本 → 点击「翻译并解析」或按 `Ctrl + Enter`
3. **追问**：结果下方展开「继续提问」，例如 *“第二句的虚拟语气再详细讲讲”*
4. **丢弃**：点击「丢弃」清空当前结果与输入，开始全新一轮

## ⚙️ 配置项说明

| 设置项 | 字段 | 说明 | 默认值 |
| --- | --- | --- | --- |
| 协议 | `provider` | `openai`（兼容协议）或 `anthropic` | `openai` |
| Base URL | `baseUrl` | API 根地址，**不含端点路径**（见下文拼接规则） | `https://api.deepseek.com` |
| API Key | `apiKey` | 服务商密钥 | 空 |
| 模型 | `model` | 模型名称 | `deepseek-chat` |
| 温度 | `temperature` | 0 ~ 1，越高越随机 | `0.7` |
| 思考深度 | `reasoningEffort` | `default` / `low` / `medium` / `high` | `default`（不发送该参数） |
| 系统提示词 | `systemPrompt` | 控制输出的结构与语言 | 内置四段式模板 |

### Base URL 拼接规则

后端会在 Base URL 末尾直接追加端点路径：

- OpenAI 兼容协议 → `{base_url}/chat/completions`
- Anthropic 协议 → `{base_url}/v1/messages`

因此：

| 服务商 | Base URL 应填 | 说明 |
| --- | --- | --- |
| DeepSeek | `https://api.deepseek.com` | 官方即不含 `/v1` |
| OpenAI | `https://api.openai.com/v1` | 需要带 `/v1` |
| Anthropic | `https://api.anthropic.com` | `/v1/messages` 由程序拼接 |
| 本地服务（如 Ollama） | `http://127.0.0.1:11434/v1` | 使用其 OpenAI 兼容端点 |

### 常见服务商速查

| 服务商 | 协议 | Base URL | 模型示例 |
| --- | --- | --- | --- |
| DeepSeek | OpenAI 兼容 | `https://api.deepseek.com` | `deepseek-chat` |
| OpenAI | OpenAI 兼容 | `https://api.openai.com/v1` | `gpt-4o-mini` |
| 通义千问 | OpenAI 兼容 | `https://dashscope.aliyuncs.com/compatible-mode/v1` | `qwen-plus` |
| Kimi（月之暗面） | OpenAI 兼容 | `https://api.moonshot.cn/v1` | `moonshot-v1-8k` |
| 智谱 GLM | OpenAI 兼容 | `https://open.bigmodel.cn/api/paas/v4` | `glm-4-flash` |
| Claude | Anthropic | `https://api.anthropic.com` | `claude-sonnet-4-5` |

> 💡 「思考深度」仅在模型支持时生效：OpenAI 协议映射到 `reasoning_effort`，Anthropic 协议映射到 extended thinking（预算 low≈1024 / medium≈4096 / high≈8192 tokens）。不支持的模型可能忽略或报错，请先「测试连接」。

## 🏗 工作原理

```
┌─────────────── 前端 (Vue) ────────────────┐      ┌──────────── Rust 后端 ────────────┐
│ HomeView / SettingsView                   │      │                                   │
│   ↓ invoke("chat_stream")                 │ ───► │ llm.rs: reqwest POST (SSE 流式)   │
│ useChat.ts 监听 Tauri 事件:                │ ◄─── │   emit("llm-chunk", {delta})     │
│   llm-chunk / llm-done / llm-error        │      │   emit("llm-done" / "llm-error")  │
│   按 requestId 匹配，增量拼接到回答        │      │ config.rs: 读写本地 config.json   │
└───────────────────────────────────────────┘      └───────────────────────────────────┘
```

- 每次流式请求都带有唯一 `requestId`，前端据此过滤事件，避免串台
- 追问时自动组装上下文：【原始文本】+【最近一轮结果】+ 当前问题
- 请求超时上限 600 秒；连接测试超时 60 秒

## 📁 目录结构

```
Translater-Helper/
├── index.html                  # Vite HTML 入口
├── vite.config.ts              # Vite 配置
├── build-release.ps1           # 一键构建脚本（NSIS / 便携版）
├── src/                        # ── 前端（Vue 3 + TS）──
│   ├── main.ts                 # 应用入口
│   ├── App.vue                 # 外壳：顶栏 + 视图切换 + 缺 Key 提示条
│   ├── types.ts                # AppConfig / ChatMessage / StreamPayload 等类型
│   ├── views/
│   │   ├── HomeView.vue        # 主界面：输入框、结果流、追问面板
│   │   └── SettingsView.vue    # 设置页：API 表单、测试连接、提示词编辑
│   ├── components/
│   │   └── MarkdownView.vue    # Markdown 安全渲染（marked + DOMPurify）
│   ├── composables/
│   │   └── useChat.ts          # 全局会话状态 + 流式事件监听 + 配置缓存
│   └── styles/main.css         # 全局样式（暗色主题）
└── src-tauri/                  # ── 后端（Rust / Tauri 2）──
    ├── tauri.conf.json         # 窗口尺寸、标识符、打包配置
    ├── Cargo.toml
    └── src/
        ├── lib.rs              # Tauri Builder，注册 3 个 command
        ├── config.rs           # 配置持久化 + 默认系统提示词
        └── llm.rs              # chat_stream：OpenAI / Anthropic 双协议 SSE 解析
```

### Tauri Commands

| Command | 说明 |
| --- | --- |
| `get_config` | 读取本地配置（不存在则返回默认值） |
| `save_config` | 保存配置到本地文件 |
| `chat_stream` | 发起流式对话，通过事件回传增量内容 |

## 💾 数据存储

配置以明文 JSON 保存在系统配置目录（Tauri `app_config_dir`）：

```
Windows:  %APPDATA%\com.translater.helper\config.json
Linux:    ~/.config/com.translater.helper/config.json
macOS:    ~/Library/Application Support/com.translater.helper/config.json
```

## 🔒 隐私与安全

- **API Key 仅存本地**：不会上传到任何第三方服务器；但以明文保存，请勿将配置文件分享给他人
- **本机发起请求**：请求直连你填写的 API，或按系统代理设置转发；应用自身不提供中转服务器
- **无历史记录**：应用不持久化任何翻译内容，关闭即消失
- **渲染消毒**：模型返回的 Markdown 经 DOMPurify 过滤后再插入 DOM

## ❓常见问题

<details>
<summary><b>Linux AppImage 界面能显示，但按钮和输入框没有反应</b></summary>

Linux 版本在 GTK / WebKit 初始化前默认设置进程级
`WEBKIT_DISABLE_DMABUF_RENDERER=1`，规避部分 Wayland / NVIDIA 环境下的
WebKitGTK 渲染异常（包括启动协议错误或界面不更新）。

只设置这一个兼容变量，不检测显卡、不强制选择 GTK 后端，也不禁用合成绘制；
不修改系统配置。已显式设置该变量时保留原值，可用
`WEBKIT_DISABLE_DMABUF_RENDERER=0` 恢复 DMA-BUF 渲染。

AppImage 的 GTK 启动脚本可能选择 X11 / XWayland，本应用不再覆盖该选择。
修改源码需重新构建才生效，旧版 AppImage 的启动策略不会自动改变。
Xvfb 自动测试覆盖隔离 X11 的设置页交互，不代表真实 Wayland 下的画面更新已通过。
</details>

<details>
<summary><b>启动后提示“尚未配置 API Key”</b></summary>

首次使用需在设置页完成配置并保存。已配置仍提示时，尝试重启应用。
</details>

<details>
<summary><b>测试连接报 404</b></summary>

多为 Base URL 拼接问题：OpenAI 官方需带 `/v1`（如 `https://api.openai.com/v1`），DeepSeek 则不带。参见上方「Base URL 拼接规则」。
</details>

<details>
<summary><b>选择“思考深度”后报错</b></summary>

当前模型不支持 `reasoning_effort` 或 extended thinking 时可能报错，改回「默认」即可。
</details>

<details>
<summary><b>生成中断或超时</b></summary>

后端整体超时为 600 秒；长文本建议分段翻译。网络不稳定时可重试。
</details>

<details>
<summary><b>想自定义输出格式</b></summary>

在设置页修改「系统提示词」，按 Markdown 二级标题组织你的板块即可，「恢复默认」可随时还原。
</details>

## 🗺 Roadmap

- [ ] 多轮连续对话（完整上下文模式）
- [ ] 翻译历史导出（Markdown / JSON）
- [ ] 划词取词 / 全局快捷键唤起
- [ ] macOS / Linux 打包支持

## 🧑‍💻 开发提示

- 推荐 IDE：[VS Code](https://code.visualstudio.com/) + [Vue - Official](https://marketplace.visualstudio.com/items?itemName=Vue.volar) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
- 修改窗口标题/尺寸：`src-tauri/tauri.conf.json` → `app.windows`
- 修改默认提示词：`src-tauri/src/config.rs` 中 `DEFAULT_SYSTEM_PROMPT` 与 `src/views/SettingsView.vue` 中同名常量需保持一致
- 新增 Tauri command：在 `lib.rs` 的 `generate_handler![]` 中注册
- Linux AppImage 界面回归：`xvfb-run -a dbus-run-session -- python3 tests/linux_appimage_smoke.py <AppImage 路径>`，使用隔离配置和测试字符，验证设置页鼠标输入、保存与返回；CI 在上传发布包前执行该检查。

---

*版本 1.0.4 · 使用 Tauri 2 构建*
