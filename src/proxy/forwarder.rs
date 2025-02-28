use std::net::TcpStream;
use std::io::{Write, Read};

pub fn forward_http_request(host: String, buffer: &[u8], mut client_stream: TcpStream) {

    // if cached


    println!("Forwarding HTTP request to: {}", host);

    match TcpStream::connect((host.as_str(), 80)) {
        Ok(mut server_stream) => {
            if let Err(e) = server_stream.write_all(buffer) {
                println!("Failed to send request to server: {}", e);
                return;
            }

            let mut server_response_buffer = [0u8; 4096];
            match server_stream.read(&mut server_response_buffer) {
                Ok(response_size) => {
                    if let Err(e) = client_stream.write_all(&server_response_buffer[..response_size]) {
                        println!("Failed to forward response: {}", e);
                    }
                }
                Err(e) => println!("Failed to read server response: {}", e),
            }
        }
        Err(e) => println!("Failed to connect to real server: {}", e),
    }
}
