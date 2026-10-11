//! Standalone, bounded HTTP metrics for the exporter process (not the TUI).
use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::time::{Duration, Instant};

const HEADER_LIMIT: usize = 8192;

fn response(request: &[u8], uptime: f64, requests: u64) -> String {
    let first = request.split(|b| *b == b'\n').next().unwrap_or_default();
    let line = String::from_utf8_lossy(first);
    let parts: Vec<_> = line.split_whitespace().collect();
    let (status, kind, body) = if parts.len() != 3 || !matches!(parts[2], "HTTP/1.0" | "HTTP/1.1") {
        ("400 Bad Request", "text/plain", "bad request\n".into())
    } else if parts[0] != "GET" {
        ("405 Method Not Allowed", "text/plain", "GET only\n".into())
    } else {
        match parts[1] {
            "/healthz" | "/readyz" => ("200 OK", "text/plain", "ok\n".into()),
            "/metrics" => (
                "200 OK",
                "text/plain; version=0.0.4",
                format!(
                    "# HELP opsec_demon_exporter_uptime_seconds Exporter process uptime.\n\
                 # TYPE opsec_demon_exporter_uptime_seconds gauge\n\
                 opsec_demon_exporter_uptime_seconds {uptime:.3}\n\
                 # HELP opsec_demon_exporter_requests_total Complete HTTP requests received.\n\
                 # TYPE opsec_demon_exporter_requests_total counter\n\
                 opsec_demon_exporter_requests_total {requests}\n"
                ),
            ),
            _ => ("404 Not Found", "text/plain", "not found\n".into()),
        }
    };
    format!(
        "HTTP/1.1 {status}\r\nContent-Type: {kind}\r\nContent-Length: {}\r\nConnection: close\r\nX-Content-Type-Options: nosniff\r\n\r\n{body}",
        body.len()
    )
}

fn read_header(stream: &mut TcpStream) -> io::Result<Vec<u8>> {
    let mut request = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(io::Error::new(io::ErrorKind::TimedOut, "header timeout"));
        }
        stream.set_read_timeout(Some(remaining))?;
        let mut buffer = [0; 512];
        let read = stream.read(&mut buffer)?;
        if read == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "incomplete header",
            ));
        }
        request.extend_from_slice(&buffer[..read]);
        if request.len() > HEADER_LIMIT {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "header too large",
            ));
        }
        if request.windows(4).any(|w| w == b"\r\n\r\n") {
            return Ok(request);
        }
    }
}

pub fn serve(address: &str) -> io::Result<()> {
    let address: SocketAddr = address
        .parse()
        .map_err(|_| io::Error::other("--metrics-server requires a numeric IP:PORT"))?;
    let listener = TcpListener::bind(address)?;
    eprintln!("[METRICS] exporter listening on {}", listener.local_addr()?);
    let started = Instant::now();
    let mut requests = 0u64;
    for connection in listener.incoming() {
        let mut stream = connection?;
        stream.set_write_timeout(Some(Duration::from_secs(2)))?;
        let reply = match read_header(&mut stream) {
            Ok(request) => {
                requests = requests.saturating_add(1);
                response(&request, started.elapsed().as_secs_f64(), requests)
            }
            Err(_) => response(b"", started.elapsed().as_secs_f64(), requests),
        };
        // A client disconnect must not kill the exporter.
        let _ = stream.write_all(reply.as_bytes());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn metrics_are_actual_values_with_prometheus_types() {
        let result = response(b"GET /metrics HTTP/1.1\r\n\r\n", 3.5, 7);
        assert!(result.starts_with("HTTP/1.1 200"));
        assert!(result.contains("opsec_demon_exporter_uptime_seconds 3.500"));
        assert!(result.contains("opsec_demon_exporter_requests_total 7"));
        let (header, body) = result.split_once("\r\n\r\n").unwrap();
        assert!(header.contains(&format!("Content-Length: {}", body.len())));
    }
    #[test]
    fn health_methods_and_unknown_paths_are_distinct() {
        assert!(response(b"GET /healthz HTTP/1.1\r\n\r\n", 0.0, 1).ends_with("ok\n"));
        assert!(response(b"POST /metrics HTTP/1.1\r\n\r\n", 0.0, 1).contains("405"));
        assert!(response(b"GET /unknown HTTP/1.0\r\n\r\n", 0.0, 1).contains("404"));
        assert!(response(b"nonsense", 0.0, 1).contains("400"));
    }
}
