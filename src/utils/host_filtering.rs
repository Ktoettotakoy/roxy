use std::collections::HashSet;
use std::sync::{Arc, Mutex};

/// Thread-safe blacklist using Arc + Mutex
pub struct Blacklist {
    blocked: Arc<Mutex<HashSet<String>>>,
}

impl Blacklist {
    /// Creates a new Blacklist instance with some default blocked hosts
    pub fn new() -> Self {
        let blocked = HashSet::from([
            // "httpforever.com".to_string(),
            "example.com".to_string(),
        ]);

        Self {
            blocked: Arc::new(Mutex::new(blocked)),
        }
    }

    /// Checks if a host is blacklisted
    pub fn has(&self, host: &str) -> bool {
        let blocked = self.blocked.lock().unwrap();
        blocked.iter().any(|blocked_host| host.contains(blocked_host))
    }

    /// Adds a host to the blacklist
    pub fn add_host(&self, host: &str) {
        let mut blocked = self.blocked.lock().unwrap();
        blocked.insert(host.to_string());
        println!("Added '{}' to blacklist", host);
    }

    /// Removes a host from the blacklist
    pub fn remove_host(&self, host: &str) -> bool {
        let mut blocked = self.blocked.lock().unwrap();
        if blocked.remove(host) {
            println!("Removed '{}' from blacklist", host);
            true
        } else {
            println!("'{}' was not in the blacklist", host);
            false
        }
    }

    /// Display all blocked hosts
    pub fn list_hosts(&self) {
        let blocked = self.blocked.lock().unwrap();
        if blocked.is_empty() {
            println!("🔹 Blacklist is empty.");
        } else {
            println!("🚫 Blacklisted hosts:");
            for host in blocked.iter() {
                println!(" - {}", host);
            }
        }
    }
}
