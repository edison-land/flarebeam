use axum::{
    body::Body,
    extract::{Request, State},
    http::{header, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{any, post},
    Json, Router,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    time::Instant,
};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct ProxyState {
    pub pin: String,
    pub session_token: String,
    pub target_port: u16,
    pub client: Client,
    pub failed_attempts: Arc<Mutex<HashMap<String, (u32, Instant)>>>,
    pub paired: Arc<AtomicBool>,
    pub request_count: Arc<AtomicUsize>,
    pub is_public: bool,
}

#[derive(Deserialize)]
pub struct VerifyPinPayload {
    pub pin: String,
}

#[derive(Serialize)]
pub struct VerifyPinResponse {
    pub ok: bool,
    pub error: Option<String>,
}

const PIN_PAGE_HTML: &str = r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no">
  <title>FlareBeam · 安全访问验证</title>
  <style>
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body {
      background-color: #09090b;
      color: #fafafa;
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
      min-height: 100vh;
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      padding: 24px;
      -webkit-font-smoothing: antialiased;
    }
    .card {
      width: 100%;
      max-width: 380px;
      background: #18181b;
      border: 1px solid rgba(255, 255, 255, 0.1);
      border-radius: 16px;
      padding: 32px 24px;
      box-shadow: 0 20px 40px rgba(0, 0, 0, 0.6);
      text-align: center;
    }
    .badge {
      display: inline-flex;
      align-items: center;
      gap: 6px;
      padding: 4px 10px;
      background: rgba(243, 128, 32, 0.12);
      border: 1px solid rgba(243, 128, 32, 0.3);
      color: #f38020;
      border-radius: 999px;
      font-size: 12px;
      font-weight: 500;
      margin-bottom: 20px;
      letter-spacing: 0.3px;
    }
    .dot {
      width: 6px;
      height: 6px;
      background: #f38020;
      border-radius: 50%;
      box-shadow: 0 0 8px #f38020;
    }
    h1 {
      font-size: 20px;
      font-weight: 600;
      letter-spacing: -0.3px;
      margin-bottom: 8px;
    }
    p {
      font-size: 13px;
      color: #a1a1aa;
      line-height: 1.5;
      margin-bottom: 28px;
    }
    .pin-grid {
      display: flex;
      justify-content: center;
      gap: 12px;
      margin-bottom: 24px;
    }
    .pin-input {
      width: 54px;
      height: 64px;
      background: #27272a;
      border: 1px solid rgba(255, 255, 255, 0.15);
      border-radius: 10px;
      color: #fff;
      font-size: 28px;
      font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
      font-weight: 600;
      text-align: center;
      outline: none;
      transition: all 0.15s ease;
      -webkit-appearance: none;
    }
    .pin-input:focus {
      border-color: #f38020;
      box-shadow: 0 0 0 3px rgba(243, 128, 32, 0.25);
      background: #18181b;
    }
    .error-msg {
      font-size: 13px;
      color: #ef4444;
      margin-bottom: 16px;
      min-height: 18px;
    }
    .submit-btn {
      width: 100%;
      height: 44px;
      background: #f38020;
      color: #fff;
      border: none;
      border-radius: 10px;
      font-size: 14px;
      font-weight: 500;
      cursor: pointer;
      transition: background 0.15s ease;
    }
    .submit-btn:hover { background: #e06f15; }
    .submit-btn:disabled { opacity: 0.5; cursor: not-allowed; }
    .footer {
      margin-top: 24px;
      font-size: 11px;
      color: #71717a;
    }
  </style>
</head>
<body>
  <div class="card">
    <div class="badge">
      <div class="dot"></div>
      FlareBeam Zero-Trust Proxy
    </div>
    <h1>请输入访问 PIN 码</h1>
    <p>该服务已被电脑端加密保护<br>请输入电脑屏幕上显示的 4 位动态数字</p>

    <div class="pin-grid">
      <input class="pin-input" type="tel" maxlength="1" pattern="[0-9]*" inputmode="numeric" autofocus autocomplete="off">
      <input class="pin-input" type="tel" maxlength="1" pattern="[0-9]*" inputmode="numeric" autocomplete="off">
      <input class="pin-input" type="tel" maxlength="1" pattern="[0-9]*" inputmode="numeric" autocomplete="off">
      <input class="pin-input" type="tel" maxlength="1" pattern="[0-9]*" inputmode="numeric" autocomplete="off">
    </div>

    <div class="error-msg" id="error"></div>
    <button class="submit-btn" id="submitBtn" disabled>验证进入</button>

    <div class="footer">
      由 Cloudflare Tunnel + FlareBeam Native Proxy 强力驱动
    </div>
  </div>

  <script>
    const inputs = Array.from(document.querySelectorAll('.pin-input'));
    const btn = document.getElementById('submitBtn');
    const errorEl = document.getElementById('error');

    function checkComplete() {
      const code = inputs.map(i => i.value).join('');
      btn.disabled = code.length !== 4;
      if (code.length === 4) {
        verify(code);
      }
    }

    inputs.forEach((input, idx) => {
      input.addEventListener('input', (e) => {
        const val = e.target.value.replace(/[^0-9]/g, '');
        e.target.value = val ? val[val.length - 1] : '';
        if (e.target.value && idx < inputs.length - 1) {
          inputs[idx + 1].focus();
        }
        checkComplete();
      });

      input.addEventListener('keydown', (e) => {
        if (e.key === 'Backspace' && !input.value && idx > 0) {
          inputs[idx - 1].focus();
          inputs[idx - 1].value = '';
          checkComplete();
        }
      });
    });

    async function verify(code) {
      btn.disabled = true;
      btn.innerText = '正在验证...';
      errorEl.innerText = '';

      try {
        const res = await fetch('/flarebeam-api/verify-pin', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ pin: code })
        });
        const data = await res.json();
        if (data.ok) {
          btn.innerText = '验证成功，正在进入...';
          setTimeout(() => {
            window.location.reload();
          }, 400);
        } else {
          errorEl.innerText = data.error || 'PIN 码错误，请重新核对';
          inputs.forEach(i => i.value = '');
          inputs[0].focus();
          btn.innerText = '验证进入';
          btn.disabled = true;
        }
      } catch (err) {
        errorEl.innerText = '连接服务器失败，请稍后重试';
        btn.innerText = '验证进入';
        btn.disabled = false;
      }
    }

    btn.addEventListener('click', () => {
      const code = inputs.map(i => i.value).join('');
      if (code.length === 4) verify(code);
    });
  </script>
</body>
</html>"#;

pub fn create_proxy_router(state: ProxyState) -> Router {
    Router::new()
        .route("/flarebeam-api/verify-pin", post(handle_verify_pin))
        .fallback(any(handle_proxy_fallback))
        .with_state(state)
}

async fn handle_verify_pin(
    State(state): State<ProxyState>,
    Json(payload): Json<VerifyPinPayload>,
) -> Response {
    let now = Instant::now();
    let mut lock = state.failed_attempts.lock().await;

    // Check if currently locked out due to excessive failed attempts
    if let Some((attempts, lock_time)) = lock.get_mut("pin_session") {
        if *attempts >= 5 {
            let elapsed = now.duration_since(*lock_time).as_secs();
            if elapsed < 300 {
                let remaining = 300 - elapsed;
                return (
                    StatusCode::TOO_MANY_REQUESTS,
                    Json(VerifyPinResponse {
                        ok: false,
                        error: Some(format!("连续输错 5 次，安全锁定中（剩余 {} 秒）", remaining)),
                    }),
                )
                    .into_response();
            } else {
                // Lock expired, reset attempt count
                *attempts = 0;
            }
        }
    }

    if payload.pin.trim() == state.pin.trim() {
        // Success: reset failed attempts counter
        lock.remove("pin_session");
        state.paired.store(true, Ordering::SeqCst);
        let cookie_header = format!(
            "flarebeam_session={}; Path=/; HttpOnly; SameSite=Lax; Max-Age=14400",
            state.session_token
        );
        let body = Json(VerifyPinResponse {
            ok: true,
            error: None,
        });

        let mut resp = body.into_response();
        if let Ok(hv) = HeaderValue::from_str(&cookie_header) {
            resp.headers_mut().insert(header::SET_COOKIE, hv);
        }
        resp
    } else {
        // Increment failed attempts and trigger lockout if limit reached
        let entry = lock.entry("pin_session".to_string()).or_insert((0, now));
        entry.0 += 1;
        entry.1 = now;

        let error_msg = if entry.0 >= 5 {
            "连续输错 5 次，已触发安全锁定 5 分钟".to_string()
        } else {
            format!("PIN 码错误，还可尝试 {} 次", 5 - entry.0)
        };

        (
            StatusCode::UNAUTHORIZED,
            Json(VerifyPinResponse {
                ok: false,
                error: Some(error_msg),
            }),
        )
            .into_response()
    }
}

async fn handle_proxy_fallback(
    State(state): State<ProxyState>,
    req: Request,
) -> Response {
    // If public mode, pass through directly
    if !state.is_public {
        // Check session cookie
        let mut authenticated = false;
        if let Some(cookie_val) = req.headers().get(header::COOKIE) {
            if let Ok(s) = cookie_val.to_str() {
                for pair in s.split(';') {
                    let mut parts = pair.trim().splitn(2, '=');
                    if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
                        if k == "flarebeam_session" && v == state.session_token {
                            authenticated = true;
                            break;
                        }
                    }
                }
            }
        }

        if !authenticated {
            return (StatusCode::UNAUTHORIZED, Html(PIN_PAGE_HTML)).into_response();
        }
    }

    // Authenticated or public: Proxy to 127.0.0.1:<target_port>
    state.request_count.fetch_add(1, Ordering::Relaxed);
    state.paired.store(true, Ordering::SeqCst);

    let (parts, body) = req.into_parts();
    let path_and_query = parts
        .uri
        .path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or("/");
    let target_url = format!("http://127.0.0.1:{}{}", state.target_port, path_and_query);

    let mut req_builder = state.client.request(parts.method, &target_url);

    // Forward headers (except hop-by-hop headers and host)
    for (name, val) in parts.headers.iter() {
        let name_lower = name.as_str();
        if name_lower != "host"
            && name_lower != "connection"
            && name_lower != "keep-alive"
            && name_lower != "transfer-encoding"
        {
            req_builder = req_builder.header(name, val);
        }
    }
    req_builder = req_builder.header(header::HOST, format!("127.0.0.1:{}", state.target_port));

    // Convert axum body to reqwest body
    let body_bytes = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(b) => b,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, "Failed to read request body").into_response();
        }
    };
    req_builder = req_builder.body(body_bytes);

    match req_builder.send().await {
        Ok(upstream_resp) => {
            let status = StatusCode::from_u16(upstream_resp.status().as_u16())
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

            let mut response_builder = Response::builder().status(status);

            for (k, v) in upstream_resp.headers().iter() {
                // Don't forward transfer-encoding or content-length if streaming
                if k != header::TRANSFER_ENCODING {
                    response_builder = response_builder.header(k, v);
                }
            }

            match upstream_resp.bytes().await {
                Ok(bytes) => response_builder
                    .body(Body::from(bytes))
                    .unwrap_or_else(|_| (StatusCode::INTERNAL_SERVER_ERROR).into_response()),
                Err(_) => (StatusCode::BAD_GATEWAY, "Failed to read upstream response").into_response(),
            }
        }
        Err(e) => {
            let err_html = format!(
                r#"<!DOCTYPE html>
<html>
<head><title>FlareBeam · 本地服务连接失败</title></head>
<body style="background:#09090b;color:#f4f4f5;font-family:system-ui;padding:40px;text-align:center;">
  <h2 style="color:#ef4444;margin-bottom:12px;">502 无法连接本地开发服务</h2>
  <p style="color:#a1a1aa;margin-bottom:20px;">FlareBeam 代理正常，但未能连通本地 127.0.0.1:{}</p>
  <code style="background:#18181b;padding:8px 16px;border-radius:6px;border:1px solid #27272a;display:inline-block;color:#f87171;">{}</code>
  <p style="color:#71717a;font-size:12px;margin-top:20px;">请确认本地 dev server (如 npm run dev) 正在该端口运行中</p>
</body>
</html>"#,
                state.target_port, e
            );
            (StatusCode::BAD_GATEWAY, Html(err_html)).into_response()
        }
    }
}
