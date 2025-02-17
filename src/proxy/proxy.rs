use std::net::{TcpListener, TcpStream};
use std::io::{Read,Write};
use std::thread;

pub fn listen_to_all_traffic_on(port: u16) {
    let host = "0.0.0.0"; // listen on all interfaces
    // let host = "127.0.0.1";

    // Bind to the specified port and handle incoming connections
    let listener = TcpListener::bind((host, port)).unwrap();
    println!("Listening on port {}...", port);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection!");

                // Create a thread that terminates when the passed function returns
                thread::spawn(move || handle_client_traffic(stream));
            }
            Err(e) => {
                println!("Connection error: {}", e);
            }
        }
    }
}

fn handle_client_traffic(mut client_stream: TcpStream) {
    let mut client_request_buffer = [0u8; 4096]; // Initialize buffer

    match client_stream.read(&mut client_request_buffer) {
        Ok(bytes_read) => {
            if bytes_read == 0 {
                println!("Client disconnected.");
                return;
            }

            let request_str = String::from_utf8_lossy(&client_request_buffer[..bytes_read]);
            println!("\nPeeked HTTP request:\n{}", request_str);

            if let Some(mut server_stream) = send_request_to_host_80(&request_str, &client_request_buffer[..bytes_read]) {
                send_response_to_client(&mut client_stream, &mut server_stream);
            }
        }
        Err(e) => {
            println!("Failed to read from stream: {}", e);
        }
    }
}

/// sends request from client (browser, my machine) to host (desired webpage)
///
/// Returns an open `TcpStream` to the server if the connection is successful.
fn send_request_to_host_80(request_str: &str, buffer: &[u8]) -> Option<TcpStream> {
    if let Some(host) = extract_host(request_str) {
        println!("Forwarding request from client to Roxy to: {}", host);

        // Attempt to connect to the web server
        match TcpStream::connect((host.as_str(), 80)) {
            Ok(mut server_stream) => {
                // Send request to the server
                if let Err(e) = server_stream.write_all(buffer) {
                    println!("Failed to send request to server: {}", e);
                    return None;
                }
                Some(server_stream)
            }
            Err(e) => {
                println!("Failed to connect to the real server: {}", e);
                None
            }
        }
    } else {
        println!("Failed to extract host from HTTP header.");
        None
    }
}

/// sends response from server (webpage on the Internet) to client (browser, my machine)
fn send_response_to_client(client_stream: &mut TcpStream, server_stream: &mut TcpStream) {
    let mut server_response_buffer = [0u8; 4096];

    match server_stream.read(&mut server_response_buffer) {
        Ok(response_size) => {
            if response_size == 0 {
                println!("Server closed the connection.");
                return;
            }

            if let Err(e) = client_stream.write_all(&server_response_buffer[..response_size]) {
                println!("Failed to forward response to client: {}", e);
            } else {
                println!("Response successfully forwarded from server to Roxy to client.");
            }
        }
        Err(e) => {
            println!("Failed to read server response: {}", e);
        }
    }
}

/// Extracts the Host header from an HTTP request
fn extract_host(request: &str) -> Option<String> {
    for line in request.lines() {
        if line.to_lowercase().starts_with("host:") {
            return Some(line[5..].trim().to_string());
        }
    }
    None
}
