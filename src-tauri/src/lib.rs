pub mod port_scanner;
pub mod proxy;
pub mod tunnel;

use port_scanner::{check_port_healthy, scan_active_dev_ports, DiscoveredPort};
use proxy::{create_proxy_router, ProxyState};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
};
use tokio::{net::TcpListener, sync::Mutex};
use tunnel::{start_quick_tunnel, TunnelHandle};

#[derive(Default)]
pub struct AppState {
    pub session: Mutex<Option<ActiveSession>>,
}

pub struct ActiveSession {
    pub tunnel: TunnelHandle,
    pub shutdown_tx: tokio::sync::oneshot::Sender<()>,
    pub target_port: u16,
    pub proxy_port: u16,
    pub pin: Option<String>,
    pub url: String,
    pub is_public: bool,
    pub paired: Arc<AtomicBool>,
    pub request_count: Arc<AtomicUsize>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PreviewInfo {
    pub url: String,
    pub pin: Option<String>,
    pub target_port: u16,
    pub proxy_port: u16,
    pub is_public: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PreviewStatus {
    pub is_active: bool,
    pub url: Option<String>,
    pub pin: Option<String>,
    pub target_port: Option<u16>,
    pub local_alive: bool,
    pub paired: bool,
    pub request_count: usize,
    pub is_public: bool,
}

#[tauri::command]
async fn scan_ports() -> Result<Vec<DiscoveredPort>, String> {
    Ok(scan_active_dev_ports().await)
}

#[tauri::command]
async fn set_window_expanded(window: tauri::WebviewWindow, expanded: bool) -> Result<(), String> {
    let target_width = if expanded { 760.0 } else { 360.0 };
    let target_height = 520.0;
    window
        .set_size(tauri::Size::Logical(tauri::LogicalSize {
            width: target_width,
            height: target_height,
        }))
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn start_preview(
    state: tauri::State<'_, AppState>,
    target_port: u16,
    is_public: bool,
) -> Result<PreviewInfo, String> {
    // 1. Verify target port is actually running locally
    if !check_port_healthy(target_port).await {
        return Err(format!(
            "本地端口 127.0.0.1:{} 未开启或拒绝连接。请先启动本地开发服务器（如 npm run dev）！",
            target_port
        ));
    }

    // Stop existing session if any
    let mut lock = state.session.lock().await;
    if let Some(mut existing) = lock.take() {
        let _ = existing.tunnel.child.kill().await;
        let _ = existing.shutdown_tx.send(());
    }

    // 2. Bind local proxy listener on random port
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| format!("绑定本地代理端口失败: {}", e))?;
    let proxy_port = listener
        .local_addr()
        .map_err(|e| format!("获取代理端口失败: {}", e))?
        .port();

    // 3. Generate 4-digit PIN and session token
    let pin_code = if is_public {
        None
    } else {
        let mut rng = rand::thread_rng();
        let num: u32 = rng.gen_range(1000..=9999);
        Some(num.to_string())
    };

    let session_token: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    let paired = Arc::new(AtomicBool::new(false));
    let request_count = Arc::new(AtomicUsize::new(0));

    let proxy_state = ProxyState {
        pin: pin_code.clone().unwrap_or_default(),
        session_token,
        target_port,
        client: reqwest::Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| format!("构建 HTTP 客户端失败: {}", e))?,
        failed_attempts: Arc::new(Mutex::new(HashMap::new())),
        paired: paired.clone(),
        request_count: request_count.clone(),
        is_public,
    };

    let router = create_proxy_router(proxy_state);
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    // Spawn Axum server with graceful shutdown
    tokio::spawn(async move {
        let _ = axum::serve(listener, router)
            .with_graceful_shutdown(async {
                let _ = shutdown_rx.await;
            })
            .await;
    });

    // 4. Launch Cloudflare Quick Tunnel pointing to PROXY_PORT
    let tunnel = match start_quick_tunnel(proxy_port).await {
        Ok(t) => t,
        Err(err) => {
            let _ = shutdown_tx.send(());
            return Err(err);
        }
    };

    let url = tunnel.url.clone();

    let info = PreviewInfo {
        url: url.clone(),
        pin: pin_code.clone(),
        target_port,
        proxy_port,
        is_public,
    };

    *lock = Some(ActiveSession {
        tunnel,
        shutdown_tx,
        target_port,
        proxy_port,
        pin: pin_code,
        url,
        is_public,
        paired,
        request_count,
    });

    Ok(info)
}

#[tauri::command]
async fn stop_preview(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut lock = state.session.lock().await;
    if let Some(mut sess) = lock.take() {
        let _ = sess.tunnel.child.kill().await;
        let _ = sess.shutdown_tx.send(());
    }
    Ok(())
}

#[tauri::command]
async fn get_preview_status(state: tauri::State<'_, AppState>) -> Result<PreviewStatus, String> {
    let lock = state.session.lock().await;
    if let Some(sess) = lock.as_ref() {
        let local_alive = check_port_healthy(sess.target_port).await;
        Ok(PreviewStatus {
            is_active: true,
            url: Some(sess.url.clone()),
            pin: sess.pin.clone(),
            target_port: Some(sess.target_port),
            local_alive,
            paired: sess.paired.load(Ordering::SeqCst),
            request_count: sess.request_count.load(Ordering::Relaxed),
            is_public: sess.is_public,
        })
    } else {
        Ok(PreviewStatus {
            is_active: false,
            url: None,
            pin: None,
            target_port: None,
            local_alive: false,
            paired: false,
            request_count: 0,
            is_public: false,
        })
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            scan_ports,
            set_window_expanded,
            start_preview,
            stop_preview,
            get_preview_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
