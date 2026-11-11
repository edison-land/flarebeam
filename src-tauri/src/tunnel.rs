use std::{
    path::PathBuf,
    process::Stdio,
    time::Duration,
};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{Child, Command},
    sync::oneshot,
    time::{sleep, timeout},
};

pub fn find_cloudflared_binary() -> Result<PathBuf, String> {
    let candidate_paths = [
        PathBuf::from("/opt/homebrew/bin/cloudflared"),
        PathBuf::from("/usr/local/bin/cloudflared"),
    ];

    for path in candidate_paths.iter() {
        if path.exists() {
            return Ok(path.clone());
        }
    }

    if let Ok(output) = std::process::Command::new("which").arg("cloudflared").output() {
        if output.status.success() {
            let p = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !p.is_empty() {
                return Ok(PathBuf::from(p));
            }
        }
    }

    Err("系统未检测到 cloudflared 二进制程序。请确保通过 Homebrew (brew install cloudflared) 安装或放置在系统 PATH 中。".to_string())
}

pub struct TunnelHandle {
    pub url: String,
    pub child: Child,
}

pub async fn start_quick_tunnel(proxy_port: u16) -> Result<TunnelHandle, String> {
    let binary = find_cloudflared_binary()?;

    let mut cmd = Command::new(binary);
    cmd.arg("tunnel")
        .arg("--config")
        .arg("/dev/null")
        .arg("--url")
        .arg(format!("http://127.0.0.1:{}", proxy_port))
        .stdout(Stdio::null())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| format!("启动 cloudflared 失败: {}", e))?;

    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "无法捕获 cloudflared stderr 流".to_string())?;

    let (url_tx, url_rx) = oneshot::channel::<String>();

    // Spawn background drain task to keep reading stderr so pipe never closes (prevents SIGPIPE)
    tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();
        let mut url_sender = Some(url_tx);

        while let Ok(Some(line)) = reader.next_line().await {
            // Find trycloudflare URL
            if line.contains(".trycloudflare.com") {
                if let Some(tx) = url_sender.take() {
                    for word in line.split_whitespace() {
                        let cleaned = word.trim_matches(|c: char| !c.is_alphanumeric() && c != ':' && c != '/' && c != '.' && c != '-');
                        if cleaned.starts_with("https://") && cleaned.ends_with(".trycloudflare.com") {
                            let _ = tx.send(cleaned.to_string());
                            break;
                        }
                    }
                }
            }
        }
    });

    // Wait up to 25 seconds for the URL to be parsed
    let url = match timeout(Duration::from_secs(25), url_rx).await {
        Ok(Ok(u)) => u,
        Ok(Err(_)) => {
            let _ = child.kill().await;
            return Err("cloudflared 进程异常退出，未输出公网 URL".to_string());
        }
        Err(_) => {
            let _ = child.kill().await;
            return Err("等待 Cloudflare 边缘节点分配 Quick Tunnel 超时（25秒）".to_string());
        }
    };

    // Give Cloudflare Edge 1.8 seconds to finish QUIC connection registration
    // This prevents Error 1033 when accessing immediately
    sleep(Duration::from_millis(1800)).await;

    Ok(TunnelHandle { url, child })
}
