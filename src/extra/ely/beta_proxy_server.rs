use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub struct ProxyGuard {
    running: Arc<AtomicBool>,
}

impl Drop for ProxyGuard {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

pub fn start_beta_proxy(port: u16) -> Option<ProxyGuard> {
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = Arc::clone(&running);

    let listener = match TcpListener::bind(format!("127.0.0.1:{}", port)) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind skin proxy on port {}: {}", port, e);
            return None;
        }
    };

    thread::spawn(move || {
        while running_clone.load(Ordering::Relaxed) {
            listener.set_nonblocking(true).ok();
            
            if let Ok((stream, _)) = listener.accept() {
                stream.set_read_timeout(Some(Duration::from_secs(2))).ok();
                thread::spawn(move || handle_proxy_client(stream));
            }
            
            thread::sleep(Duration::from_millis(50));
        }
    });

    Some(ProxyGuard { running })
}

fn handle_proxy_client(mut client: TcpStream) {
    let mut buffer = [0u8; 4096];
    let bytes_read = match client.read(&mut buffer) {
        Ok(n) if n > 0 => n,
        _ => return,
    };

    let request_str = String::from_utf8_lossy(&buffer[..bytes_read]);
    let first_line = request_str.lines().next().unwrap_or("");

    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 2 {
        return;
    }

    let url = parts[1];

    if url.contains("/MinecraftSkins/") {
        if let Some(filename) = url.split("/MinecraftSkins/").nth(1) {
            let username = filename.split('?').next().unwrap_or(filename).trim_end_matches(".png");
            
            println!("[Proxy] Запрос скина для игрока: {}", username);

            let target_url = format!("https://skinsystem.ely.by/skins/{}.png", username);

            let http_client = reqwest::blocking::Client::builder()
                .user_agent("Mozilla/5.0")
                .timeout(Duration::from_secs(5))
                .build();

            if let Ok(http) = http_client {
                if let Ok(mut resp) = http.get(&target_url).send() {
                    println!("[Proxy] Ответ от Ely.by: {}", resp.status());
                    
                    if resp.status().is_success() {
                        let mut img_data = Vec::new();
                        if resp.read_to_end(&mut img_data).is_ok() {
                            let header = format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: image/png\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                                img_data.len()
                            );
                            let _ = client.write_all(header.as_bytes());
                            let _ = client.write_all(&img_data);
                            return;
                        }
                    }
                }
            }
        }
    }

    let not_found = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
    let _ = client.write_all(not_found.as_bytes());
}
