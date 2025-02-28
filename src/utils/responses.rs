use std::io::Write;
use std::net::TcpStream;

/// Sends a generic HTTP response
fn send_response(client_stream: &mut TcpStream, status_line: &str, body: &str) {
    let response = format!(
        "HTTP/1.1 {}\r\n\
        Content-Length: {}\r\n\
        Content-Type: text/plain\r\n\
        Connection: close\r\n\
        \r\n\
        {}",
        status_line,
        body.len(),
        body
    );

    let _ = client_stream.write_all(response.as_bytes());
    let _ = client_stream.flush();
}

/// Sends a `403 Forbidden` response
pub fn send_403_forbidden(client_stream: &mut TcpStream) {
    send_response(client_stream, "403 Forbidden", "Access Denied: Blacklisted");
}
