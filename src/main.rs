use roxy::cli::console::out_starting_message;
use roxy::proxy::proxy::listen_to_all_traffic_on;
// use roxy::client::simple_client::send_get_every_5_seconds_to_port;
use std::thread;

fn main() {
    out_starting_message();

    // Start the proxy server thread
    let proxy_thread = thread::spawn(move || listen_to_all_traffic_on(6505));

    // Start a client thread for testing, not yet implemented correctly
    // let client_thread = thread::spawn(move || send_get_every_5_seconds_to_port(80));

    // (they won't finish, since they run indefinitely)
    proxy_thread.join().unwrap();
    // client_thread.join().unwrap();
}
