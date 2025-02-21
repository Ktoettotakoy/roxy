use roxy::cli::console::out_starting_message;
use roxy::proxy::listener::start_proxy;
// use roxy::client::simple_client::send_get_every_5_seconds_to_port;
use std::thread;

fn main() {
    out_starting_message();

    // Start the proxy server thread
    let proxy_thread = thread::spawn(move || start_proxy(6505));

    // Start a client thread for testing, not yet implemented correctly
    // let client_thread = thread::spawn(move || send_get_every_5_seconds_to_port(80));

    // (they won't finish, since they run indefinitely)
    proxy_thread.join().unwrap();
    // client_thread.join().unwrap();
}
