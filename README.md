# triSig

一个轻量级的桌面红绿灯状态指示器，在屏幕右上角（可配置）显示红、黄、绿三色信号灯，支持通过 HTTP API 远程切换状态和闪烁效果。

## 适用场景

- **Claude Code / 自动化工具状态指示**：在 Claude Code 的 hook 或脚本中通过 HTTP 请求切换灯色，直观展示当前工作状态（如处理中 / 等待输入 / 空闲）
- **CI/CD 流水线状态监控**：构建脚本在关键节点发送请求，桌面即时反馈流水线进度
- **本地服务健康检查**：定时请求 health endpoint，服务异常时红灯闪烁
- **番茄钟 / 专注模式**：通过脚本控制灯色切换，作为免打扰的视觉信号

## 系统架构

```
┌─────────────┐   HTTP GET    ┌──────────────────┐   Tauri Event   ┌─────────────────┐
│  外部工具     │ ────────────> │  tiny_http Server │ ─────────────> │  Vue 3 前端      │
│ (curl/脚本)  │              │  (port 53789+)    │               │  (透明悬浮窗)     │
└─────────────┘               └──────────────────┘               └─────────────────┘
                                     │
                                     │ 嵌入
                                     ▼
                              ┌──────────────────┐
                              │   Tauri v2 App   │
                              │  (Rust + WebView) │
                              └──────────────────┘
```

- **前端 (Vue 3 + Vite)**：渲染透明无边框窗口，红黄绿三盏灯，支持常亮和闪烁模式
- **HTTP Server (tiny_http)**：内嵌于 Tauri 应用，监听 LAN IP，接收 GET 请求并转发为 Tauri 事件
- **Tauri v2 托盘**：系统托盘菜单，可切换显示器、切换左右角、查看服务地址、退出应用

窗口特性：
- 100×180px 透明无边框悬浮窗，始终置顶，不在任务栏显示
- 默认位于主显示器右上角，可通过托盘菜单切换至左上角或其他显示器
- 支持 macOS 多显示器，自动选择尺寸最大的显示器作为默认位置

## API

HTTP Server 启动后监听局域网地址，端口从 `53789` 起自动分配。

```
GET /?color={red|yellow|green}&interval={milliseconds}
```

| 参数 | 必填 | 说明 |
|------|------|------|
| `color` | 是 | 灯色：`red`、`yellow`、`green` |
| `interval` | 是 | 闪烁间隔（毫秒），传 `0` 表示常亮 |

示例：

```bash
# 常亮绿灯
curl "http://192.168.1.100:53789/?color=green&interval=0"

# 红灯闪烁 (500ms 间隔)
curl "http://192.168.1.100:53789/?color=red&interval=500"

# 黄灯常亮
curl "http://127.0.0.1:53789/?color=yellow&interval=0"
```

托盘菜单中可直接复制 LAN 和 Local 两个地址。

## 构建

### 前置依赖

- [Rust](https://www.rust-lang.org/) (stable)
- [Node.js](https://nodejs.org/) >= 18
- [pnpm](https://pnpm.io/)
- macOS: Xcode Command Line Tools
- Windows: Microsoft Visual Studio C++ Build Tools
- Linux: `webkit2gtk-4.1`, `libayatana-appindicator3` 等系统库（详见 [Tauri 文档](https://v2.tauri.app/start/prerequisites/)）

### 开发模式

```bash
# 安装依赖
pnpm install

# 启动开发服务器 (含热更新)
pnpm tauri dev
```

### 生产构建

```bash
pnpm tauri build
```

构建产物位于 `src-tauri/target/release/bundle/`，包含对应平台的安装包（macOS 为 `.dmg`，Windows 为 `.msi`/`.exe`，Linux 为 `.deb`/`.AppImage`）。

## 支持平台

| 平台 | 状态 |
|------|------|
| macOS | 完全支持 |
| Windows | 完全支持 |
| Linux | 完全支持 |

基于 [Tauri v2](https://v2.tauri.app/)，利用各平台原生 WebView 渲染（macOS 使用 WebKit，Windows 使用 WebView2，Linux 使用 WebKitGTK）。

## 技术栈

- **桌面框架**：Tauri v2 (Rust)
- **前端**：Vue 3 + Vite
- **HTTP 服务**：tiny_http 0.12
- **事件通信**：Tauri Event System (Rust → WebView)
