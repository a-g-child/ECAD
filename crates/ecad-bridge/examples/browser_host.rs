//! Local browser test/development transport. Production desktop uses Tauri IPC instead.
use ecad_bridge::{DesktopBridge, Request};
use std::{
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    path::Path,
    time::Duration,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let project = std::env::var_os("ECAD_UI_PROJECT");
    let profile_path = std::env::var_os("ECAD_UI_PROFILES")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            project.as_ref().and_then(|p| {
                Path::new(p)
                    .parent()
                    .map(|p| p.join("company-profiles.json"))
            })
        });
    let mut bridge = if let Some(path) = project {
        {
            let path = Path::new(&path);
            if path.exists() {
                DesktopBridge::from_path(path)?
            } else {
                let mut bridge = DesktopBridge::demo()?;
                bridge.save_as(path)?;
                bridge
            }
        }
    } else {
        DesktopBridge::demo()?
    };
    if let Some(path) = profile_path {
        bridge.configure_profile_store(path);
    }
    let listener = TcpListener::bind("127.0.0.1:3838")?;
    println!("Browser harness listening on 127.0.0.1:3838; only the local Vite origin may call it");
    for stream in listener.incoming() {
        let mut stream = stream?;
        stream.set_read_timeout(Some(Duration::from_secs(3)))?;
        let reply = handle(&mut stream, &mut bridge).unwrap_or_else(
            |e| serde_json::json!({"error":{"code":"transport","message":e.to_string()}}),
        );
        let text = serde_json::to_vec(&reply)?;
        write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\nCache-Control: no-store\r\n\r\n",
            text.len()
        )?;
        stream.write_all(&text)?;
    }
    Ok(())
}
fn handle(
    stream: &mut TcpStream,
    bridge: &mut DesktopBridge,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut reader = BufReader::new(stream);
    let mut first = String::new();
    reader.read_line(&mut first)?;
    if first.trim() != "POST /bridge HTTP/1.1" {
        return Err("Only POST /bridge is supported".into());
    }
    let mut size = None;
    let mut origin = None;
    let mut json = false;
    let mut total = first.len();
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line)?;
        total += n;
        if n == 0 || total > 16384 {
            return Err("Invalid request headers".into());
        }
        if line == "\r\n" {
            break;
        }
        let Some((name, value)) = line.split_once(':') else {
            return Err("Invalid header".into());
        };
        match name.to_ascii_lowercase().as_str() {
            "content-length" => {
                if size.is_some() {
                    return Err("Duplicate content length".into());
                }
                size = Some(value.trim().parse::<usize>()?);
            }
            "content-type" => json = value.trim() == "application/json",
            "origin" => origin = Some(value.trim().to_string()),
            "transfer-encoding" => return Err("Chunked requests are unsupported".into()),
            _ => {}
        }
    }
    if !json
        || !matches!(
            origin.as_deref(),
            Some("http://127.0.0.1:1420" | "http://localhost:1420" | "http://127.0.0.1:1421")
        )
    {
        return Err("Only JSON requests from the local UI are allowed".into());
    }
    let size = size
        .filter(|s| *s <= 262144)
        .ok_or("Missing or excessive content length")?;
    let mut body = vec![0; size];
    reader.read_exact(&mut body)?;
    let request: Request = serde_json::from_slice(&body)?;
    Ok(match bridge.handle(request) {
        Ok(response) => serde_json::json!({"ok":response}),
        Err(error) => serde_json::json!({"error":error}),
    })
}
