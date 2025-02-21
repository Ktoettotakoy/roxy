use std::net::TcpStream;
use std::io::Write;
use std::thread;
use std::sync::{Arc,Mutex};
use crate::utils::parsing::extract_host;

pub fn handle_https_tunnel(request_str: &str, mut client_stream: TcpStream) {
    if let Some(host) = extract_host(request_str) {
        println!("Handling CONNECT request to {}", host);

        if let Ok(server_stream) = TcpStream::connect(&host) {
            client_stream.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n").unwrap();
            println!("Established tunnel to {}", &host);

            tunnel_data(client_stream, server_stream);
        } else {
            println!("Failed to connect to HTTPS server");
        }
    }
}

fn tunnel_data(client: TcpStream, server: TcpStream) {
    let client = Arc::new(Mutex::new(client));
    let server = Arc::new(Mutex::new(server));

    let client_clone = Arc::clone(&client);
    let server_clone = Arc::clone(&server);

    let client_to_server = thread::spawn(move || {
        let mut client_lock = client_clone.lock().unwrap();
        let mut server_lock = server_clone.lock().unwrap();
        std::io::copy(&mut *client_lock, &mut *server_lock).unwrap();
    });

    let client_clone = Arc::clone(&client);
    let server_clone = Arc::clone(&server);

    let server_to_client = thread::spawn(move || {
        let mut client_lock = client_clone.lock().unwrap();
        let mut server_lock = server_clone.lock().unwrap();
        std::io::copy(&mut *server_lock, &mut *client_lock).unwrap();
    });

    client_to_server.join().unwrap();
    server_to_client.join().unwrap();
}
