mod server;

use std::path::PathBuf;
use std::sync::Mutex;

use arboard::Clipboard;
use serde::{Deserialize, Serialize};
use server::ServerInfo;
use tauri::{
    menu::{CheckMenuItem, MenuBuilder, MenuItem, SubmenuBuilder},
    tray::TrayIconBuilder,
    AppHandle, Emitter, LogicalPosition, Manager, Monitor, WebviewWindow, Wry,
};
use tauri_plugin_notification::NotificationExt;

#[derive(Clone, Copy)]
enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Corner {
    fn as_str(self) -> &'static str {
        match self {
            Corner::TopLeft => "top-left",
            Corner::TopRight => "top-right",
            Corner::BottomLeft => "bottom-left",
            Corner::BottomRight => "bottom-right",
        }
    }

    fn parse(s: &str) -> Self {
        match s {
            "top-left" => Corner::TopLeft,
            "bottom-left" => Corner::BottomLeft,
            "bottom-right" => Corner::BottomRight,
            _ => Corner::TopRight,
        }
    }
}

struct AppState {
    corner: Mutex<Corner>,
    monitor_index: Mutex<usize>,
    server_info: Mutex<Option<ServerInfo>>,
    transparency: Mutex<f64>,
    muted: Mutex<bool>,
    mute_item: Mutex<Option<CheckMenuItem<Wry>>>,
}

#[derive(Clone, Serialize)]
struct TransparencyEvent {
    transparency: f64,
}

#[derive(Clone, Serialize)]
struct MuteEvent {
    muted: bool,
}

#[derive(Clone, Serialize)]
struct AppStateSnapshot {
    transparency: f64,
    muted: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
struct Config {
    corner: String,
    monitor_index: Option<usize>,
    transparency: f64,
    muted: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            corner: "top-right".to_string(),
            monitor_index: None,
            transparency: 1.0,
            muted: false,
        }
    }
}

const WINDOW_W: f64 = 100.0;
const WINDOW_H: f64 = 180.0;
const MARGIN: f64 = 20.0;

fn config_path(app: &AppHandle) -> Option<PathBuf> {
    let home = app.path().home_dir().ok()?;
    Some(home.join(".config").join("trisig.json"))
}

fn load_config(app: &AppHandle) -> Config {
    config_path(app)
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_config(app: &AppHandle) {
    let Some(path) = config_path(app) else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let state = app.state::<AppState>();
    let config = Config {
        corner: state.corner.lock().unwrap().as_str().to_string(),
        monitor_index: Some(*state.monitor_index.lock().unwrap()),
        transparency: *state.transparency.lock().unwrap(),
        muted: *state.muted.lock().unwrap(),
    };
    if let Ok(content) = serde_json::to_string_pretty(&config) {
        let _ = std::fs::write(path, content);
    }
}

#[tauri::command]
fn get_app_state(state: tauri::State<'_, AppState>) -> AppStateSnapshot {
    AppStateSnapshot {
        transparency: *state.transparency.lock().unwrap(),
        muted: *state.muted.lock().unwrap(),
    }
}

fn position_window(window: &WebviewWindow, monitor: &Monitor, corner: Corner) {
    let scale = monitor.scale_factor();
    let size = monitor.size();
    let pos = monitor.position();
    let logical_w = size.width as f64 / scale;
    let logical_h = size.height as f64 / scale;
    let logical_x = pos.x as f64 / scale;
    let logical_y = pos.y as f64 / scale;
    let x = match corner {
        Corner::TopLeft | Corner::BottomLeft => logical_x + MARGIN,
        Corner::TopRight | Corner::BottomRight => logical_x + logical_w - WINDOW_W - MARGIN,
    };
    let y = match corner {
        Corner::TopLeft | Corner::TopRight => logical_y + MARGIN,
        Corner::BottomLeft | Corner::BottomRight => logical_y + logical_h - WINDOW_H - MARGIN,
    };
    let _ = window.set_position(LogicalPosition::new(x, y));
}

fn apply_layout(app: &tauri::AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    let state = app.state::<AppState>();
    let corner = *state.corner.lock().unwrap();
    let idx = *state.monitor_index.lock().unwrap();
    if let Ok(monitors) = window.available_monitors() {
        if let Some(mon) = monitors.get(idx).or_else(|| monitors.first()) {
            position_window(&window, mon, corner);
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .manage(AppState {
            corner: Mutex::new(Corner::TopRight),
            monitor_index: Mutex::new(0),
            server_info: Mutex::new(None),
            transparency: Mutex::new(1.0),
            muted: Mutex::new(false),
            mute_item: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![get_app_state])
        .setup(|app| {
            let config = load_config(app.handle());

            let window = app.get_webview_window("main").expect("main window missing");
            let monitors = window.available_monitors().unwrap_or_default();

            let initial_idx = config
                .monitor_index
                .filter(|&i| i < monitors.len())
                .unwrap_or_else(|| {
                    monitors
                        .iter()
                        .enumerate()
                        .max_by_key(|(_, m)| {
                            let s = m.size();
                            u64::from(s.width) * u64::from(s.height)
                        })
                        .map(|(i, _)| i)
                        .unwrap_or(0)
                });

            {
                let state = app.state::<AppState>();
                *state.corner.lock().unwrap() = Corner::parse(&config.corner);
                *state.monitor_index.lock().unwrap() = initial_idx;
                *state.transparency.lock().unwrap() = config.transparency;
                *state.muted.lock().unwrap() = config.muted;
            }

            apply_layout(app.handle());

            let server_info = server::start(app.handle().clone());
            *app.state::<AppState>().server_info.lock().unwrap() = server_info.clone();

            let mut menu_builder = MenuBuilder::new(app);
            if let Some(info) = &server_info {
                let lan = MenuItem::with_id(
                    app,
                    "info:lan",
                    format!("局域网  http://{}:{}", info.lan_ip, info.port),
                    true,
                    None::<&str>,
                )?;
                let local = MenuItem::with_id(
                    app,
                    "info:local",
                    format!("本地 http://127.0.0.1:{}", info.port),
                    true,
                    None::<&str>,
                )?;
                menu_builder = menu_builder.item(&lan).item(&local).separator();
            } else {
                let down = MenuItem::with_id(
                    app,
                    "info:down",
                    "HTTP 服务不可用",
                    false,
                    None::<&str>,
                )?;
                menu_builder = menu_builder.item(&down).separator();
            }

            let mute_item = CheckMenuItem::with_id(
                app,
                "mute:toggle",
                "静音",
                true,
                config.muted,
                None::<&str>,
            )?;
            *app.state::<AppState>().mute_item.lock().unwrap() = Some(mute_item.clone());

            let mut display_submenu = SubmenuBuilder::new(app, "移动到显示器");
            for (i, mon) in monitors.iter().enumerate() {
                let label = mon
                    .name()
                    .cloned()
                    .filter(|n| !n.is_empty())
                    .unwrap_or_else(|| format!("显示器 {}", i + 1));
                let item = MenuItem::with_id(
                    app,
                    format!("monitor:{i}"),
                    label,
                    true,
                    None::<&str>,
                )?;
                display_submenu = display_submenu.item(&item);
            }
            let display_submenu = display_submenu.build()?;

            let mut corner_sub = SubmenuBuilder::new(app, "调整位置");
            let corners: [(&str, &str); 4] = [
                ("corner:top-left", "左上角"),
                ("corner:top-right", "右上角"),
                ("corner:bottom-left", "左下角"),
                ("corner:bottom-right", "右下角"),
            ];
            for (id, label) in &corners {
                let item =
                    MenuItem::with_id(app, *id, *label, true, None::<&str>)?;
                corner_sub = corner_sub.item(&item);
            }
            let corner_sub = corner_sub.build()?;

            let mut trans_sub = SubmenuBuilder::new(app, "背景透明度");
            let trans_opts: [(&str, f64); 5] = [
                ("100%", 1.0),
                ("80%", 0.8),
                ("60%", 0.6),
                ("40%", 0.4),
                ("20%", 0.2),
            ];
            for (label, val) in &trans_opts {
                let item = MenuItem::with_id(
                    app,
                    format!("transparency:{val}"),
                    *label,
                    true,
                    None::<&str>,
                )?;
                trans_sub = trans_sub.item(&item);
            }
            let trans_sub = trans_sub.build()?;

            let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;

            let menu = menu_builder
                .item(&mute_item)
                .item(&display_submenu)
                .item(&trans_sub)
                .item(&corner_sub)
                .separator()
                .item(&quit)
                .build()?;

            let _tray = TrayIconBuilder::with_id("main-tray")
                .icon(app.default_window_icon().cloned().expect("missing icon"))
                .menu(&menu)
                .on_menu_event(|app, event| {
                    let id = event.id.as_ref();
                    if id == "quit" {
                        app.exit(0);
                        return;
                    }
                    let state = app.state::<AppState>();
                    if id == "mute:toggle" {
                        let new_muted = {
                            let mut m = state.muted.lock().unwrap();
                            *m = !*m;
                            *m
                        };
                        if let Some(item) = state.mute_item.lock().unwrap().as_ref() {
                            let _ = item.set_checked(new_muted);
                        }
                        let _ = app.emit("traffic-mute", MuteEvent { muted: new_muted });
                        save_config(app);
                        return;
                    }
                    if let Some(idx_str) = id.strip_prefix("monitor:") {
                        if let Ok(idx) = idx_str.parse::<usize>() {
                            *state.monitor_index.lock().unwrap() = idx;
                            apply_layout(app);
                            save_config(app);
                        }
                        return;
                    }
                    // Handle network address copy
                    if let Some(addr) = id.strip_prefix("info:") {
                        let info = state.server_info.lock().unwrap();
                        if let Some(ref info) = *info {
                            let url = match addr {
                                "lan" => format!("http://{}:{}", info.lan_ip, info.port),
                                "local" => format!("http://127.0.0.1:{}", info.port),
                                _ => return,
                            };
                            if Clipboard::new()
                                .and_then(|mut c| c.set_text(url))
                                .is_ok()
                            {
                                let _ = app
                                    .notification()
                                    .builder()
                                    .title("triSig")
                                    .body("网络地址已复制")
                                    .show();
                            }
                        }
                        return;
                    }
                    if let Some(val_str) = id.strip_prefix("transparency:") {
                        if let Ok(val) = val_str.parse::<f64>() {
                            *state.transparency.lock().unwrap() = val;
                            let _ = app.emit(
                                "traffic-transparency",
                                TransparencyEvent { transparency: val },
                            );
                            save_config(app);
                        }
                        return;
                    }
                    match id {
                        "corner:top-left" => {
                            *state.corner.lock().unwrap() = Corner::TopLeft;
                            apply_layout(app);
                            save_config(app);
                        }
                        "corner:top-right" => {
                            *state.corner.lock().unwrap() = Corner::TopRight;
                            apply_layout(app);
                            save_config(app);
                        }
                        "corner:bottom-left" => {
                            *state.corner.lock().unwrap() = Corner::BottomLeft;
                            apply_layout(app);
                            save_config(app);
                        }
                        "corner:bottom-right" => {
                            *state.corner.lock().unwrap() = Corner::BottomRight;
                            apply_layout(app);
                            save_config(app);
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
