use std::net::TcpStream;
use std::io::Read;
use std::sync::Arc;

use super::http::forward_http_request;
use super::https::handle_https_tunnel;
use crate::utils::parsing::extract_host;

use crate::utils::responses::send_403_forbidden;

use crate::utils::host_filtering::Blacklist;
use crate::proxy::cache::HttpCache;


pub fn handle_client_connection(mut client_stream: TcpStream, blacklist: Arc<Blacklist>, cache: Arc<HttpCache>) {
    let mut buffer = [0u8; 8192];

    match client_stream.read(&mut buffer) {
        Ok(bytes_read) => {
            if bytes_read == 0 {
                println!("Client disconnected.");
                return;
            }

            let request_str = String::from_utf8_lossy(&buffer[..bytes_read]);
            println!("\nPeeked HTTP/S request:\n{}", request_str);

            // Extract host first
            match extract_host(&request_str) {
                Some(host) => {
                    // Check blacklist
                    if blacklist.has(&host) {
                        println!("Host '{}' is blacklisted", host);
                        send_403_forbidden(&mut client_stream);

                        // Close client connection
                        let _ = client_stream.shutdown(std::net::Shutdown::Both);
                        return;
                    }

                    // Process based on request type
                    if request_str.starts_with("CONNECT") {
                        let _ = handle_https_tunnel(&request_str, client_stream);
                    } else {
                        forward_http_request(host, &buffer[..bytes_read], client_stream, Arc::clone(&cache));
                    }
                },
                None => {
                    println!("Failed to extract host from request");
                    // Maybe send a 400 Bad Request response here
                    let _ = client_stream.shutdown(std::net::Shutdown::Both);
                }
            }
        },
        Err(e) => {
            println!("Failed to read from stream: {}", e);
            let _ = client_stream.shutdown(std::net::Shutdown::Both);
        }
    }
}
