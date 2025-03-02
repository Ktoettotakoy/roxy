use std::net::TcpStream;
use std::io::{Write, Read};
use std::sync::Arc;

use crate::utils::parsing::parse_http_request;
use crate::utils::parsing::parse_http_response;
use crate::proxy::cache::HttpCache;

pub fn forward_http_request(host: String, buffer: &[u8], mut client_stream: TcpStream, cache: Arc<HttpCache>) {
    let request_str = String::from_utf8_lossy(buffer);
    let request_headers = parse_http_request(&request_str).unwrap().headers;


    println!("Forwarding HTTP request to: {}", host);

    // Check if cached first
    if let Some(cached_entry) = cache.get(host.clone(), &request_headers) {
        println!("Cache hit for {}", host);
        client_stream.write_all(&cached_entry.response_data).unwrap();
        return;
    }

    // If not cached forward to server
    match TcpStream::connect(&host) {
        Ok(mut server_stream) => {
            if let Err(e) = server_stream.write_all(buffer) {
                println!("Failed to send request to server: {}", e);
                return;
            }

            let mut server_response_buffer = [0u8; 8192];
            match server_stream.read(&mut server_response_buffer) {
                Ok(response_size) => {
                    let response_str = String::from_utf8_lossy(&server_response_buffer[..response_size]);
                    println!("\nPeeked HTTP response:\n{}", response_str);

                    let parsed_response = parse_http_response(&response_str);
                    let response_data = server_response_buffer[..response_size].to_vec();

                    // store in cache
                    cache.put(host, response_data.clone(), parsed_response.unwrap().headers.clone());

                    if let Err(e) = client_stream.write_all(&response_data) {
                        println!("Failed to forward response: {}", e);
                    }
                }
                Err(e) => println!("Failed to read server response: {}", e),
            }
        }
        Err(e) => println!("Failed to connect to real server: {}", e),
    }
}
