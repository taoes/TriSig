use std::net::UdpSocket;
use std::thread;

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tiny_http::{Method, Response, Server};

#[derive(Clone, Serialize)]
struct TrafficLight {
    color: String,
    interval: f64,
}

pub struct ServerInfo {
    pub port: u16,
    pub lan_ip: String,
}

const BASE_PORT: u16 = 53789;
const MAX_TRIES: u16 = 100;

pub fn start(app: AppHandle) -> Option<ServerInfo> {
    let mut port = BASE_PORT;
    let server = loop {
        match Server::http(("0.0.0.0", port)) {
            Ok(s) => break s,
            Err(e) => {
                if port - BASE_PORT + 1 >= MAX_TRIES {
                    eprintln!("[server] bind failed after {MAX_TRIES} tries: {e}");
                    return None;
                }
                port += 1;
            }
        }
    };
    let lan_ip = detect_lan_ip().unwrap_or_else(|| "127.0.0.1".to_string());

    thread::spawn(move || {
        for req in server.incoming_requests() {
            handle(&app, req);
        }
    });

    Some(ServerInfo { port, lan_ip })
}

fn handle(app: &AppHandle, req: tiny_http::Request) {
    if req.method() != &Method::Get {
        let _ = req.respond(
            Response::from_string("method not allowed").with_status_code(405),
        );
        return;
    }
    let (color, interval) = parse_query(req.url());
    let resp = match (color.as_deref(), interval) {
        (Some(c), Some(i)) if matches!(c, "red" | "yellow" | "green") && i >= 0.0 => {
            let payload = TrafficLight {
                color: c.to_string(),
                interval: i,
            };
            let _ = app.emit("traffic-light", payload);
            Response::from_string(format!("ok color={c} interval={i}\n"))
        }
        _ => Response::from_string(
            "usage: GET /?color={red|yellow|green}&interval={milliseconds, 0 = solid}\n",
        )
        .with_status_code(400),
    };
    let _ = req.respond(resp);
}

fn parse_query(url: &str) -> (Option<String>, Option<f64>) {
    let q = url.split_once('?').map(|(_, q)| q).unwrap_or("");
    let mut color = None;
    let mut interval = None;
    for pair in q.split('&') {
        let Some((k, v)) = pair.split_once('=') else {
            continue;
        };
        match k {
            "color" => color = Some(v.to_string()),
            "interval" => interval = v.parse().ok(),
            _ => {}
        }
    }
    (color, interval)
}

fn detect_lan_ip() -> Option<String> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|a| a.ip().to_string())
}
