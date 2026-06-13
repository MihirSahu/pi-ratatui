use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::mpsc::Sender,
    thread,
    time::Duration,
};

pub(crate) const DEFAULT_CONTROL_ADDR: &str = "127.0.0.1:9898";
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Clone, Copy)]
pub(crate) enum ControlCommand {
    Reset,
    SetScene(ControlScene),
}

#[derive(Clone, Copy)]
pub(crate) enum ControlScene {
    Dashboard,
    Coding,
    Suit,
    Roam,
}

impl ControlScene {
    pub(crate) fn from_path(value: &str) -> Option<Self> {
        match value {
            "dashboard" | "walking" | "walk" => Some(Self::Dashboard),
            "coding" | "laptop" => Some(Self::Coding),
            "suit" | "suited" => Some(Self::Suit),
            "roam" | "roaming" => Some(Self::Roam),
            _ => None,
        }
    }
}

pub(crate) fn start(sender: Sender<ControlCommand>) -> std::io::Result<SocketAddr> {
    let listener = TcpListener::bind(DEFAULT_CONTROL_ADDR)?;
    let addr = listener.local_addr()?;

    thread::Builder::new()
        .name("pi-tui-control-api".to_owned())
        .spawn(move || {
            for stream in listener.incoming().flatten() {
                let sender = sender.clone();
                thread::spawn(move || handle_connection(stream, &sender));
            }
        })?;

    Ok(addr)
}

fn handle_connection(mut stream: TcpStream, sender: &Sender<ControlCommand>) {
    let _ = stream.set_read_timeout(Some(CONNECTION_TIMEOUT));
    let _ = stream.set_write_timeout(Some(CONNECTION_TIMEOUT));

    let mut buffer = [0; 2048];
    let Ok(bytes_read) = stream.read(&mut buffer) else {
        return;
    };

    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    let Some((method, path)) = request_line(&request) else {
        write_json(stream, 400, r#"{"ok":false,"error":"bad request"}"#);
        return;
    };

    match (method, path) {
        ("GET", "/health") => write_json(stream, 200, r#"{"ok":true}"#),
        ("GET", "/animations") => write_json(
            stream,
            200,
            r#"{"animations":["dashboard","walking","coding","suit","roam"]}"#,
        ),
        ("POST", "/reset") => send_command(stream, sender, ControlCommand::Reset),
        ("POST", path) => {
            if let Some(scene) = path
                .strip_prefix("/scene/")
                .or_else(|| path.strip_prefix("/animation/"))
                .and_then(ControlScene::from_path)
            {
                send_command(stream, sender, ControlCommand::SetScene(scene));
            } else {
                write_json(stream, 404, r#"{"ok":false,"error":"unknown endpoint"}"#);
            }
        }
        _ => write_json(stream, 404, r#"{"ok":false,"error":"unknown endpoint"}"#),
    }
}

fn request_line(request: &str) -> Option<(&str, &str)> {
    let mut parts = request.lines().next()?.split_whitespace();
    Some((parts.next()?, parts.next()?))
}

fn send_command(stream: TcpStream, sender: &Sender<ControlCommand>, command: ControlCommand) {
    if sender.send(command).is_ok() {
        write_json(stream, 202, r#"{"ok":true}"#);
    } else {
        write_json(
            stream,
            503,
            r#"{"ok":false,"error":"dashboard unavailable"}"#,
        );
    }
}

fn write_json(mut stream: TcpStream, status: u16, body: &str) {
    let reason = match status {
        200 => "OK",
        202 => "Accepted",
        400 => "Bad Request",
        404 => "Not Found",
        503 => "Service Unavailable",
        _ => "OK",
    };

    let response = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes());
}
