mod server;

use std::sync::Mutex;

use tauri::{
    menu::{MenuBuilder, MenuItem, SubmenuBuilder},
    tray::TrayIconBuilder,
    LogicalPosition, Manager, Monitor, WebviewWindow,
};

#[derive(Clone, Copy)]
enum Corner {
    TopLeft,
    TopRight,
}

struct AppState {
    corner: Mutex<Corner>,
    monitor_index: Mutex<usize>,
}

const WINDOW_W: f64 = 100.0;
const MARGIN: f64 = 20.0;

fn position_window(window: &WebviewWindow, monitor: &Monitor, corner: Corner) {
    let scale = monitor.scale_factor();
    let size = monitor.size();
    let pos = monitor.position();
    let logical_w = size.width as f64 / scale;
    let logical_x = pos.x as f64 / scale;
    let logical_y = pos.y as f64 / scale;
    let x = match corner {
        Corner::TopLeft => logical_x + MARGIN,
        Corner::TopRight => logical_x + logical_w - WINDOW_W - MARGIN,
    };
    let y = logical_y + MARGIN;
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
        .manage(AppState {
            corner: Mutex::new(Corner::TopRight),
            monitor_index: Mutex::new(0),
        })
        .setup(|app| {
            let window = app.get_webview_window("main").expect("main window missing");
            let monitors = window.available_monitors().unwrap_or_default();

            let largest_idx = monitors
                .iter()
                .enumerate()
                .max_by_key(|(_, m)| {
                    let s = m.size();
                    u64::from(s.width) * u64::from(s.height)
                })
                .map(|(i, _)| i)
                .unwrap_or(0);
            *app.state::<AppState>().monitor_index.lock().unwrap() = largest_idx;

            apply_layout(app.handle());

            let server_info = server::start(app.handle().clone());

            let mut menu_builder = MenuBuilder::new(app);
            if let Some(info) = &server_info {
                let lan = MenuItem::with_id(
                    app,
                    "info:lan",
                    format!("LAN  http://{}:{}", info.lan_ip, info.port),
                    false,
                    None::<&str>,
                )?;
                let local = MenuItem::with_id(
                    app,
                    "info:local",
                    format!("Local http://127.0.0.1:{}", info.port),
                    false,
                    None::<&str>,
                )?;
                menu_builder = menu_builder.item(&lan).item(&local).separator();
            } else {
                let down = MenuItem::with_id(
                    app,
                    "info:down",
                    "HTTP server unavailable",
                    false,
                    None::<&str>,
                )?;
                menu_builder = menu_builder.item(&down).separator();
            }

            let mut display_submenu = SubmenuBuilder::new(app, "Move to Display");
            for (i, mon) in monitors.iter().enumerate() {
                let label = mon
                    .name()
                    .cloned()
                    .filter(|n| !n.is_empty())
                    .unwrap_or_else(|| format!("Display {}", i + 1));
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

            let corner_left =
                MenuItem::with_id(app, "corner:left", "Top-Left Corner", true, None::<&str>)?;
            let corner_right =
                MenuItem::with_id(app, "corner:right", "Top-Right Corner", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

            let menu = menu_builder
                .item(&display_submenu)
                .separator()
                .item(&corner_left)
                .item(&corner_right)
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
                    if let Some(idx_str) = id.strip_prefix("monitor:") {
                        if let Ok(idx) = idx_str.parse::<usize>() {
                            *state.monitor_index.lock().unwrap() = idx;
                            apply_layout(app);
                        }
                        return;
                    }
                    match id {
                        "corner:left" => {
                            *state.corner.lock().unwrap() = Corner::TopLeft;
                            apply_layout(app);
                        }
                        "corner:right" => {
                            *state.corner.lock().unwrap() = Corner::TopRight;
                            apply_layout(app);
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
