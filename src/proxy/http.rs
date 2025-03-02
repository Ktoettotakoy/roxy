use std::net::TcpStream;
use std::io::{Write, Read};
use std::sync::Arc;
use std::time::Instant;  // Import timing

use crate::utils::parsing::parse_http_request;
use crate::utils::parsing::parse_http_response;
use crate::proxy::cache::HttpCache;

pub fn forward_http_request(host: String, buffer: &[u8], mut client_stream: TcpStream, cache: Arc<HttpCache>) {
    let request_str = String::from_utf8_lossy(buffer);
    let request_headers = parse_http_request(&request_str).unwrap().headers;

    println!("Forwarding HTTP request to: {}", host);

    let start_total = Instant::now();  // Start total timing
    // ✅ Measure Cache Lookup Time
    let start_cache = Instant::now();
    if let Some(cached_entry) = cache.get(&host, &request_headers) {
        let cache_time = start_cache.elapsed();
        println!("Cache hit for {} (lookup time: {:.2?})", host, cache_time);

        let start_send_cache = Instant::now();
        if let Err(e) = client_stream.write_all(&cached_entry.response_data) {
            println!("Failed to forward cached response: {}", e);
        }
        let send_cache_time = start_send_cache.elapsed();
        println!("Cached response sent in {:.2?}", send_cache_time);

        let total_time = start_total.elapsed();
        println!("Total request time (cache hit): {:.2?}", total_time);
        return;
    }
    let cache_time = start_cache.elapsed();
    println!("Cache miss (lookup time: {:.2?})", cache_time);

    // ✅ Measure Request Forwarding Time
    let start_forward = Instant::now();
    match TcpStream::connect(&host) {
        Ok(mut server_stream) => {
            if let Err(e) = server_stream.write_all(buffer) {
                println!("Failed to send request to server: {}", e);
                return;
            }

            let forward_time = start_forward.elapsed();
            println!("Request forwarded in {:.2?}", forward_time);

            let mut server_response_buffer = [0u8; 8192 * 64];

            let start_response = Instant::now();
            match server_stream.read(&mut server_response_buffer) {
                Ok(response_size) => {
                    let response_time = start_response.elapsed();
                    println!("Received response from server in {:.2?}", response_time);

                    let response_str = String::from_utf8_lossy(&server_response_buffer[..response_size]);

                    let parsed_response = parse_http_response(&response_str);
                    let response_data = server_response_buffer[..response_size].to_vec();

                    // Store in cache
                    let _ = cache.put(&host, response_data.clone(), parsed_response.unwrap().headers.clone());

                    let start_send_client = Instant::now();
                    if let Err(e) = client_stream.write_all(&response_data) {
                        println!("Failed to forward response: {}", e);
                    }
                    let send_client_time = start_send_client.elapsed();
                    println!("Response sent to client in {:.2?}", send_client_time);
                }
                Err(e) => println!("Failed to read server response: {}", e),
            }
        }
        Err(e) => println!("Failed to connect to real server: {}", e),
    }

    let total_time = start_total.elapsed();
    println!("Total request time: {:.2?}", total_time);
}
