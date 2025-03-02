use std::thread;
use std::sync::Arc;


use roxy::cli::console::command_listener;
use roxy::proxy::listener::start_proxy;
use roxy::utils::host_filtering::Blacklist;
use roxy::proxy::cache::{HttpCache, CacheConfig};

fn main() {
    // Initialize blacklist with default banned webpages
    let blacklist = Arc::new(Blacklist::new());

    // Initialize the cache
    let cache_config = CacheConfig::default();
    let cache = match HttpCache::new(cache_config) {
        Ok(cache) => Arc::new(cache),
        Err(e) => {
            eprintln!("Failed to initialize cache: {}", e);
            return;
        }
    };

    // Create clone for command listener thread
    let blacklist_clone_cmd = Arc::clone(&blacklist);
    let cache_clone_cmd = Arc::clone(&cache);
    let command_thread = thread::spawn(move || command_listener(blacklist_clone_cmd, cache_clone_cmd));

    // Create clone for proxy thread
    let blacklist_clone_proxy = Arc::clone(&blacklist);
    let cache_clone_proxy = Arc::clone(&cache);
    let proxy_thread = thread::spawn(move || start_proxy(6505, blacklist_clone_proxy, cache_clone_proxy));

    // Wait for proxy thread to finish (which it won't since it runs indefinitely)
    proxy_thread.join().unwrap();
    command_thread.join().unwrap();
}
