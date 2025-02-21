use std::net::TcpListener;
use std::thread;
use super::handler::handle_client_connection;

pub fn start_proxy(port: u16) {
    let listener = TcpListener::bind(("0.0.0.0", port)).expect("Failed to bind to port");
    println!("Listening on port {}...", port);

    let mut connection_counter: u32 = 0;
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {}!", connection_counter);
                thread::spawn(move || handle_client_connection(stream));
                connection_counter += 1;
            }
            Err(e) => {
                println!("Connection error: {}", e);
            }
        }
    }
}
