use std::net::TcpListener;
use std::thread;
use std::sync::Arc;

use super::handler::handle_client_connection;
use crate::utils::host_filtering::Blacklist;
use crate::proxy::cache::HttpCache;

/// # Listens for incoming connections

pub fn start_proxy(port: u16, blacklist: Arc<Blacklist>, cache: Arc<HttpCache>) {
    let listener = TcpListener::bind(("0.0.0.0", port)).expect("Failed to bind to port");
    println!("Listening on port {}...", port);

    let mut connection_counter: u32 = 0;
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {}!", connection_counter);
                // Arc Clone blacklist and L1 Cache hashmap to use in thread
                let cache_clone = Arc::clone(&cache);
                let blacklist_clone = Arc::clone(&blacklist);

                thread::spawn(move || handle_client_connection(stream, blacklist_clone, cache_clone));
                connection_counter += 1;
            }
            Err(e) => {
                println!("Connection error: {}", e);
            }
        }
    }
}
