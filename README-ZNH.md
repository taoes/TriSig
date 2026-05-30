# triSig

- 英文版本： [README.md](README.md)
- 中文版本：本文件

一个轻量级的桌面交通灯状态指示器，可在屏幕右上角显示红、黄、绿信号灯（可配置）。支持通过 HTTP API 远程切换状态和闪烁效果。

## 使用场景

- **Claude Code / 自动化工具状态**：通过 HTTP 请求在 Claude Code 钩子或脚本中切换灯光颜色，以直观显示当前工作状态（例如处理 / 等待输入 / 空闲）。详见 `docs/claude_code.md`。
- **CI/CD 管道监控**：构建脚本在关键阶段发送请求，以在桌面上即时反馈流水线进度。
- **本地服务健康检查**：周期性请求健康端点——服务宕机时红灯闪烁。
- **番茄钟 / 专注模式**：通过脚本控制灯光转换，作为无干扰的视觉信号。

## 系统架构

```
┌─────────────┐   HTTP GET    ┌──────────────────┐   Tauri 事件   ┌─────────────────┐
│  外部工具    │ ────────────> │  tiny_http 服务  │ ─────────────> │  Vue 3 前端     │
│ (curl/脚本) │              │  (端口 53789+)   │               │ (透明覆盖窗口)  │
└─────────────┘                     │                            └─────────────────┘
                                    │ 嵌入式
                                    ▼
                             ┌──────────────────┐
                             │   Tauri v2 应用   │
                             │  (Rust + WebView) │
                             └──────────────────┘
```

- **前端 (Vue 3 + Vite)**：渲染一个透明无边框窗口，显示红、黄、绿灯；支持恒亮和闪烁模式。
- **HTTP 服务器 (tiny_http)**：嵌入在 Tauri 应用中，启动后监听 LAN 地址，接收 GET 请求并转发为 Tauri 事件。
- **Tauri v2 托盘**：系统托盘菜单用于切换显示、切换窗口角落、查看/复制服务器地址和退出。

窗口特性：
- 100×180px 透明无边框浮动窗口，始终置顶，不出现在任务栏。
- 默认显示在主显示器右上角；可通过托盘菜单切换到左上角或其他显示器。
- 支持 macOS 多显示器；默认自动选择最大显示器。

## API

HTTP 服务器在启动后监听 LAN 地址。端口从 `53789` 开始自动分配。

```
GET /?color={red|yellow|green}&interval={milliseconds}
```

| 参数 | 必选 | 说明 |
|------|------|------|
| `color` | 是 | 灯光颜色：`red`、`yellow`、`green` |
| `interval` | 是 | 闪烁间隔（毫秒）；使用 `0` 表示常亮 |

示例：

```bash
# 绿色常亮
curl "http://192.168.1.100:53789/?color=green&interval=0"

# 红灯闪烁（500ms 间隔）
curl "http://192.168.1.100:53789/?color=red&interval=500"

# 黄色常亮
curl "http://127.0.0.1:53789/?color=yellow&interval=0"
```

点击托盘菜单中的任何网络地址可复制到剪贴板。

## 构建

### 前置条件

- [Rust](https://www.rust-lang.org/)（stable）
- [Node.js](https://nodejs.org/) >= 18
- [pnpm](https://pnpm.io/)
- macOS：Xcode 命令行工具
- Windows：Microsoft Visual Studio C++ Build Tools
- Linux：`webkit2gtk-4.1`、`libayatana-appindicator3` 以及其他系统库（见 [Tauri 文档](https://v2.tauri.app/start/prerequisites/)）

### 开发

```bash
# 安装依赖
pnpm install

# 启动开发服务器（热重载）
pnpm tauri dev
```

### 生产构建

```bash
pnpm tauri build
```

构建产物位于 `src-tauri/target/release/bundle/`，包含平台特定安装包（macOS 为 `.dmg`，Windows 为 `.msi`/`.exe`，Linux 为 `.deb`/`.AppImage`）。

## 支持平台

| 平台 | 状态 |
|------|------|
| macOS | 完全支持 |
| Windows | 完全支持 |
| Linux | 完全支持 |

基于 [Tauri v2](https://v2.tauri.app/)，在各平台使用本地 WebView 渲染（macOS 使用 WebKit，Windows 使用 WebView2，Linux 使用 WebKitGTK）。

## 技术栈

- **桌面框架**：Tauri v2 (Rust)
- **前端**：Vue 3 + Vite
- **HTTP 服务器**：tiny_http 0.12
- **事件通信**：Tauri 事件系统 (Rust → WebView)
