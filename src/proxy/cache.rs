use std::collections::HashMap;
use std::sync::{Mutex};
use std::time::{Duration, SystemTime};

// Cache entry structure
#[derive(Clone)]
pub struct CacheEntry {
    pub response_data: Vec<u8>,
    pub headers: HashMap<String, String>,
    pub timestamp: SystemTime,
    pub max_age: Option<Duration>,
    pub last_modified: Option<String>,
}

// Cache manager
pub struct HttpCache {
    entries: Mutex<HashMap<String, CacheEntry>>,
}

impl HttpCache {
    pub fn new() -> Self {
        HttpCache {
            entries: Mutex::new(HashMap::new()),
        }
    }

    // Check if we have a valid cached response
    pub fn get(&self, host: String, request_headers: &HashMap<String, String>) -> Option<CacheEntry> {
        let entries = self.entries.lock().unwrap();

        if let Some(entry) = entries.get(&host) {
            // Print the headers (for debugging)
            // println!("Cached headers: {:?}\n", entry.headers);
            // Print body (for debugging)
            // println!("Cached body: {:?}\n", String::from_utf8_lossy(&entry.response_data));
            println!("Cache time related vars: {:?}, {:?}, {:?}", entry.timestamp, entry.max_age, entry.last_modified);

            return Some(entry.clone());
        }

        None
    }

    // Store a response in the cache
    pub fn put(&self, host: String, response_data: Vec<u8>, headers: HashMap<String, String>) {
        let mut entries = self.entries.lock().unwrap();

        // Parse caching directives from headers
        let max_age = parse_max_age(&headers);
        let last_modified = headers.get("last-modified").cloned();

        // Only cache if allowed by Cache-Control
        if let Some(cache_control) = headers.get("cache-control") {
            if cache_control.contains("no-store") {
                return; // Don't cache
            }
        }

        entries.insert(host, CacheEntry {
            response_data,
            headers,
            timestamp: SystemTime::now(),
            max_age,
            last_modified,
        });
    }
}

// Helper function to parse max-age from Cache-Control header
fn parse_max_age(headers: &HashMap<String, String>) -> Option<Duration> {
    if let Some(cache_control) = headers.get("cache-control") {
        for directive in cache_control.split(',') {
            let directive = directive.trim();
            if directive.starts_with("max-age=") {
                if let Ok(seconds) = directive[8..].parse::<u64>() {
                    return Some(Duration::from_secs(seconds));
                }
            }
        }
    }

    None
}
