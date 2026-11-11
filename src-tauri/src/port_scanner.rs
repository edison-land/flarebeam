use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, time::Duration};
use tokio::net::TcpStream;
use tokio::time::timeout;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DiscoveredPort {
    pub port: u16,
    pub name: String,
}

pub async fn scan_active_dev_ports() -> Vec<DiscoveredPort> {
    let mut candidate_procs: HashMap<u16, String> = HashMap::new();

    // 1. Gather all listening TCP ports and their process names from lsof
    if let Ok(output) = tokio::process::Command::new("lsof")
        .args(["-nP", "-iTCP", "-sTCP:LISTEN"])
        .output()
        .await
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 9 {
                let cmd = parts[0];
                let addr = parts[8];
                if let Some(pos) = addr.rfind(':') {
                    if let Ok(port) = addr[pos + 1..].parse::<u16>() {
                        let cmd_lower = cmd.to_lowercase();
                        // Filter out non-web background apps, desktop clients and proxies
                        // Also ignore ephemeral/private ports (>= 49152) and privileged system ports (< 1024)
                        if port >= 1024
                            && port < 49152
                            && !cmd_lower.contains("wechat")
                            && !cmd_lower.contains("wps")
                            && !cmd_lower.contains("clash")
                            && !cmd_lower.contains("rapportd")
                            && !cmd_lower.contains("postgres")
                            && !cmd_lower.contains("redis")
                            && !cmd_lower.contains("cloudflar")
                            && !cmd_lower.contains("antigravi")
                            && !cmd_lower.contains("onedrive")
                            && !cmd_lower.contains("google")
                            && !cmd_lower.contains("language_")
                            && port != 1420 // FlareBeam's internal Vite dev server
                            && port != 7890 // Local Proxy
                            && port != 20241 // Cloudflared metrics
                            && port != 20242
                        {
                            candidate_procs.entry(port).or_insert(cmd.to_string());
                        }
                    }
                }
            }
        }
    }

    // 2. Also ensure standard web development ports are checked
    let standard_ports = [3000, 5173, 5728, 8080, 8000, 4173, 4321, 8081, 8888, 9000, 3001, 5000];
    for &p in &standard_ports {
        candidate_procs.entry(p).or_insert("Web Server".to_string());
    }

    // 3. True HTTP Web Semantic Probe:
    // Only keep ports that actually speak HTTP!
    let client = Client::builder()
        .no_proxy()
        .timeout(Duration::from_millis(200))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap_or_default();

    let mut valid_web_services = Vec::new();

    for (port, proc_name) in candidate_procs {
        // Quick TCP check
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        if timeout(Duration::from_millis(60), TcpStream::connect(addr))
            .await
            .is_ok_and(|res| res.is_ok())
        {
            // Real HTTP Web Probe
            let url = format!("http://127.0.0.1:{}/", port);
            if let Ok(resp) = client.get(&url).send().await {
                // If the port returned an HTTP status, it is confirmed to be an HTTP web service!
                let _status = resp.status();
                let clean_name = if proc_name.contains("Python") || proc_name.contains("python") {
                    "Python".to_string()
                } else if proc_name.contains("node") || proc_name.contains("vite") || proc_name.contains("next") {
                    "Node / Vite".to_string()
                } else if proc_name.contains("bun") {
                    "Bun".to_string()
                } else if proc_name.contains("go") {
                    "Go".to_string()
                } else if proc_name.contains("cargo") || proc_name.contains("rust") {
                    "Rust".to_string()
                } else if proc_name == "Web Server" {
                    "HTTP".to_string()
                } else {
                    proc_name
                };

                valid_web_services.push(DiscoveredPort {
                    port,
                    name: clean_name,
                });
            }
        }
    }

    valid_web_services.sort_by_key(|p| p.port);
    valid_web_services
}

pub async fn check_port_healthy(port: u16) -> bool {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    timeout(Duration::from_millis(150), TcpStream::connect(addr))
        .await
        .is_ok_and(|res| res.is_ok())
}
