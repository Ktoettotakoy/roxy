use std::net::TcpStream;
use std::io::Read;
use super::forwarder::forward_http_request;
use super::tunnel::handle_https_tunnel;
use crate::utils::parsing::extract_host;

pub fn handle_client_connection(mut client_stream: TcpStream) {
    let mut buffer = [0u8; 4096];

    match client_stream.read(&mut buffer) {
        Ok(bytes_read) => {
            if bytes_read == 0 {
                println!("Client disconnected.");
                return;
            }

            let request_str = String::from_utf8_lossy(&buffer[..bytes_read]);
            println!("\nPeeked HTTP/S request:\n{}", request_str);

            // here we have to care about 2 cases:
            // we create an https tunnel
            // we have a http request
            if request_str.starts_with("CONNECT") {
                handle_https_tunnel(&request_str, client_stream);
            } else if let Some(host) = extract_host(&request_str) {
                forward_http_request(host, &buffer[..bytes_read], client_stream);
            }
        }
        Err(e) => {
            println!("Failed to read from stream: {}", e);
        }
    }
}
