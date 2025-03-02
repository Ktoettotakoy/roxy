use std::net::TcpStream;
use std::io::Write;
use std::thread;
use std::io;

use std::time::Duration;
use std::io::Read;


use crate::utils::parsing::extract_host;


pub fn handle_https_tunnel(request_str: &str, mut client_stream: TcpStream) -> io::Result<()> {
    if let Some(host) = extract_host(request_str) {
        println!("Handling CONNECT request to {}", host);

        match TcpStream::connect(&host) {
            Ok(server_stream) => {
                // Send success response to client
                client_stream.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")?;
                println!("Established tunnel to {}", &host);

                // Set non-blocking for both streams
                client_stream.set_nonblocking(true)?;
                server_stream.set_nonblocking(true)?;

                // Create separate streams for each direction
                tunnel_data(client_stream, server_stream)
            },
            Err(e) => {
                println!("Failed to connect to HTTPS server: {}", e);
                client_stream.write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n")?;
                Ok(())
            }
        }
    } else {
        println!("Invalid CONNECT request: no host found");
        client_stream.write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n")?;
        Ok(())
    }
}

fn tunnel_data(mut client_stream: TcpStream, mut server_stream: TcpStream) -> io::Result<()> {
    let mut buffer = [0; 8192];

    // Clone the streams for the two threads
    let mut client_read = client_stream.try_clone()?;
    let mut server_read = server_stream.try_clone()?;

    // Create a thread to handle client -> server data
    let client_to_server = thread::spawn(move || {
        let mut buffer = [0; 8192];
        loop {
            match client_read.read(&mut buffer) {
                Ok(0) => break, // Connection closed
                Ok(n) => {
                    if let Err(e) = server_stream.write_all(&buffer[0..n]) {
                        println!("Error writing to server: {}", e);
                        break;
                    }
                },
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10));
                    continue;
                },
                Err(e) => {
                    println!("Error reading from client: {}", e);
                    break;
                }
            }
        }
    });

    // Handle server -> client data in the current thread
    loop {
        match server_read.read(&mut buffer) {
            Ok(0) => break, // Connection closed
            Ok(n) => {
                if let Err(e) = client_stream.write_all(&buffer[0..n]) {
                    println!("Error writing to client: {}", e);
                    break;
                }
            },
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(10));
                continue;
            },
            Err(e) => {
                println!("Error reading from server: {}", e);
                break;
            }
        }
    }

    // Wait for the client-to-server thread to finish
    let _ = client_to_server.join();

    Ok(())
}
