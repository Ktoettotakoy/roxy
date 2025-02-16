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
                thread::spawn(move || handle_client_80(stream));
            }
            Err(e) => {
                println!("Connection error: {}", e);
            }
        }
    }
}

fn handle_client_80(mut stream: TcpStream) {
    let mut buffer = [0u8; 4096]; // Initialize the buffer with 4096 bytes

    // Attempt to read data from the client stream
    match stream.read(&mut buffer) {
        Ok(bytes_read) => {
            if bytes_read == 0 {
                // If nothing is read, it means the client disconnected
                println!("Client disconnected.");
                return; // Exit the thread if no data is read
            }

            let request_str = String::from_utf8_lossy(&buffer[..bytes_read]);
            println!("\nPeeked HTTP request:\n{}", request_str);

            // Extract the Host header to determine where to forward the request
            if let Some(host) = extract_host(&request_str) {
                println!("Forwarding request to: {}", host);

                // Connect to the real server (host on port 80)
                if let Ok(mut server_stream) = TcpStream::connect((host.as_str(), 80)) {
                    // Forward the request to the real server
                    server_stream.write_all(&buffer[..bytes_read]).expect("Failed to send request to the real server");

                    // Read the response from the real web server
                    let mut server_response = [0u8; 4096];
                    match server_stream.read(&mut server_response) {
                        Ok(response_size) => {
                            // Forward the server response back to the client (browser)
                            stream.write_all(&server_response[..response_size]).expect("Failed to forward response to client");
                            println!("Request forwarded back to: {}", host);
                        }
                        Err(e) => {
                            println!("Failed to read server response: {}", e);
                        }
                    }
                } else {
                    println!("Failed to connect to the real server");
                }
            }
        }
        Err(e) => {
            println!("Failed to read from stream: {}", e);
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
