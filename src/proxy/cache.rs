use std::collections::{HashMap, VecDeque};
use std::thread;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use redis::{Commands, Connection, RedisError};
use serde::{Serialize, Deserialize};

/// Represents a cached HTTP response with associated metadata.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CacheEntry {
    /// The raw response bytes
    pub response_data: Vec<u8>,

    /// HTTP headers from the original response
    pub headers: HashMap<String, String>,

    /// When this entry was added to the cache
    pub timestamp: u64,

    /// Last-Modified value from the response headers (if present)
    pub last_modified: Option<String>,

    /// ETag value from the response headers (if present)
    pub etag: Option<String>,

    /// Cache expiration time (None means no explicit expiration)
    pub expires_at: Option<u64>,
}

impl CacheEntry {
    /// Creates a new cache entry from response data and headers
    pub fn new(response_data: Vec<u8>, headers: HashMap<String, String>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Extract cache control directives
        let mut expires_at = None;
        let mut max_age = None;
        let etag = headers.get("etag").cloned();
        let last_modified = headers.get("last-modified").cloned();

        // Parse Cache-Control header
        if let Some(cache_control) = headers.get("cache-control") {
            for directive in cache_control.split(',') {
                let directive = directive.trim();
                if directive == "no-store" || directive == "no-cache" {
                    // Don't cache or validate on each use
                    return CacheEntry {
                        response_data,
                        headers,
                        timestamp: now,
                        last_modified,
                        etag,
                        expires_at: Some(now), // Expire immediately
                    };
                } else if directive.starts_with("max-age=") {
                    if let Ok(seconds) = directive[8..].parse::<u64>() {
                        max_age = Some(seconds);
                    }
                }
            }
        }

        // Parse Expires header if max-age wasn't specified
        if max_age.is_none() {
            if let Some(expires) = headers.get("expires") {

                // This is just a placeholder
                if expires != "0" && !expires.is_empty() {
                    // Set some arbitrary expiration as example
                    expires_at = Some(now + 5); // 20 seconds
                }
            }
        } else {
            // Use max-age if it was specified
            expires_at = Some(now + max_age.unwrap());
        }

        CacheEntry {
            response_data,
            headers,
            timestamp: now,
            last_modified,
            etag,
            expires_at,
        }
    }

    /// Checks if this cache entry is still valid
    pub fn is_valid(&self) -> bool {
        // ideally should work, but something is off
        if let Some(expires_at) = self.expires_at {
            println!("Expires at: {}", expires_at);
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            return now < expires_at;
        }
        // No expiration time specified, consider valid
        true
    }

    /// Checks if this entry matches the conditional request headers
    pub fn matches_conditional_headers(&self, request_headers: &HashMap<String, String>) -> bool {
        // Check If-None-Match against ETag
        if let Some(if_none_match) = request_headers.get("if-none-match") {
            if let Some(etag) = &self.etag {
                return if_none_match == etag;
            }
        }

        // Check If-Modified-Since against Last-Modified
        if let Some(if_modified_since) = request_headers.get("if-modified-since") {
            if let Some(last_modified) = &self.last_modified {
                return if_modified_since == last_modified;
            }
        }

        false
    }
}

/// Configuration for the HTTP cache
pub struct CacheConfig {
    /// Maximum size of the in-memory (L1) cache
    pub l1_max_size: usize,

    /// Default TTL for L1 cache entries (in seconds)
    pub l1_default_ttl: u64,

    /// Number of hits required to promote from L2 to L1
    pub promotion_threshold: usize,

    /// Redis connection string
    pub redis_url: String,
}

impl Default for CacheConfig {
    fn default() -> Self {
        CacheConfig {
            l1_max_size: 1000,
            l1_default_ttl: 20,
            promotion_threshold: 5,
            redis_url: "redis://127.0.0.1/".to_string(),
        }
    }
}

/// A two-level HTTP cache with L1 (in-memory) and L2 (Redis) storage
pub struct HttpCache {
    /// L1 cache entries with access count for LRU implementation
    l1_entries: Arc<RwLock<HashMap<String, (CacheEntry, usize)>>>,

    /// Track hits for potential promotion from L2 to L1
    hit_counters: Arc<RwLock<HashMap<String, usize>>>,

    /// Redis client for L2 (DB) caching
    db_client: redis::Client,

    /// Cache configuration
    config: CacheConfig,

    /// Queue for LRU eviction
    lru_queue: Arc<Mutex<VecDeque<String>>>,
}

impl HttpCache {
    /// Creates a new HttpCache with the provided configuration
    pub fn new(config: CacheConfig) -> Result<HttpCache, RedisError> {
        let client = redis::Client::open(&config.redis_url[..])?;

        // Test connection to Redis
        let mut con = client.get_connection()?;
        let _: String = con.ping()?;

        Ok(HttpCache {
            l1_entries: Arc::new(RwLock::new(HashMap::new())),
            hit_counters: Arc::new(RwLock::new(HashMap::new())),
            db_client: client,
            config,
            lru_queue: Arc::new(Mutex::new(VecDeque::new())),
        })
    }

    /// Get a connection to Redis
    fn get_connection(&self) -> Result<Connection, RedisError> {
        self.db_client.get_connection()
    }

    /// Get an item from cache (either L1 or L2)
    pub fn get(&self, host: &str, request_headers: &HashMap<String, String>) -> Option<CacheEntry> {

        // First try L1 cache (fast path)
        {
            let mut l1_entries = match self.l1_entries.write() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };

            if let Some((entry, access_count)) = l1_entries.get_mut(host) {
                // Update access count
                *access_count += 1;

                // Check if entry is still valid
                if entry.is_valid() {
                    // Check if this is a conditional request
                    if entry.matches_conditional_headers(request_headers) {
                        return Some(entry.clone());
                    }
                    println!("Returning from L1");
                    return Some(entry.clone());
                } else {
                    // Entry expired, remove from L1
                    l1_entries.remove(host);
                }
            }
        }
        // Try L2 cache
        match self.get_from_l2(host) {
            Ok(Some(entry)) => {
                // Check if entry is valid
                if !entry.is_valid() {
                    return None;
                }

                // Check if this is a conditional request
                if entry.matches_conditional_headers(request_headers) {
                    // Update hit counter for potential promotion
                    self.increment_hit_counter(host);
                    println!("Returning from L2");
                    return Some(entry);
                }

                // Update hit counter for potential promotion
                self.increment_hit_counter(host);
                println!("Returning from L2");
                Some(entry.clone())
            },
            Ok(None) => None,
            Err(e) => {
                println!("Error retrieving from L2 cache: {}", e);
                None
            }
        }
    }

    /// Get an entry from the L2 (Redis) cache
    fn get_from_l2(&self, host: &str) -> Result<Option<CacheEntry>, RedisError> {
        let mut con = self.get_connection()?;

        match con.get::<_, Option<String>>(host) {
            Ok(Some(serialized_entry)) => {
                match serde_json::from_str(&serialized_entry) {
                    Ok(entry) => {
                        // println!("\n\n\nEntry {:?}\n\n\n", entry);
                        Ok(Some(entry))
                    },
                    Err(e) => {
                        println!("Error deserializing cache entry: {}", e);
                        Ok(None)
                    }
                }
            },
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Increment the hit counter for a host and check for promotion
    fn increment_hit_counter(&self, host: &str) {
        let mut hit_counters = match self.hit_counters.write() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        let counter = hit_counters.entry(host.to_string()).or_insert(0);
        *counter += 1;

        // Check if it should be promoted to L1
        if *counter >= self.config.promotion_threshold {
            hit_counters.remove(host);

            // Clone host for the thread
            let host_str = host.to_string();
            let self_clone = self.clone();

            // Move to L1 asynchronously to not block the current request
            thread::spawn(move || {
                if let Ok(Some(entry)) = self_clone.get_from_l2(&host_str) {
                    self_clone.put_l1(&host_str, entry);
                }
            });
        }
    }

    /// Store a response in both L1 and L2 caches
    pub fn put(&self, host: &str, response_data: Vec<u8>, headers: HashMap<String, String>) -> Result<(), RedisError> {
        println!("Adding to cache...");
        let entry = CacheEntry::new(response_data, headers);

        // Skip caching if the entry is immediately expired
        if !entry.is_valid() {
            return Ok(());
        }

        // Store in cache
        self.put_l2(host, entry.clone())?;
        // self.put_l1(host, entry);

        Ok(())
    }

    /// Store a response in L2 (Redis) cache
    fn put_l2(&self, host: &str, entry: CacheEntry) -> Result<(), RedisError> {
        let mut con = self.get_connection()?;

        // Serialize the entry
        let serialized_entry = match serde_json::to_string(&entry) {
            Ok(s) => s,
            Err(e) => {
                println!("Error serializing cache entry: {}", e);
                return Ok(());
            }
        };

        // Store in Redis with TTL if available
        if let Some(expires_at) = entry.expires_at {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            if expires_at > now {
                let ttl = expires_at - now;
                let _: () = con.set_ex(host, serialized_entry, ttl)?;
            } else {
                // Don't cache if already expired
            }
        } else {
            // No expiration, use default
            let _: () = con.set(host, serialized_entry)?;
        }

        // Increment hit counter to track access frequency
        self.increment_hit_counter(host);

        // Check if this entry qualifies for promotion to L1
        {
            let hit_counters = self.hit_counters.read().unwrap();
            if let Some(&hit_count) = hit_counters.get(host) {
                if hit_count >= self.config.promotion_threshold {
                    // Move entry from L2 to L1
                    println!("Promoting {} to L1 cache", host);
                    self.put_l1(host, entry);
                }
            }
        }
        Ok(())
    }

    /// Store a response in L1 (in-memory) cache with LRU eviction
    fn put_l1(&self, host: &str, entry: CacheEntry) {
        let mut l1_entries = match self.l1_entries.write() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        let mut lru_queue = match self.lru_queue.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        // Check if we need to evict an entry
        if l1_entries.len() >= self.config.l1_max_size {
            // Evict the least recently used entry
            if let Some(oldest_host) = lru_queue.pop_front() {
                l1_entries.remove(&oldest_host);
            }
        }

        // Add to L1 cache
        l1_entries.insert(host.to_string(), (entry.clone(), 1));
        lru_queue.push_back(host.to_string());

        // Schedule removal from L1 cache after TTL
        if let Some(expires_at) = entry.expires_at {
            let host_clone = host.to_string();
            let l1_entries_clone = self.l1_entries.clone();

            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            if expires_at > now {
                let ttl = expires_at - now;

                thread::spawn(move || {
                    thread::sleep(Duration::from_secs(ttl));

                    let mut l1_entries = match l1_entries_clone.write() {
                        Ok(guard) => guard,
                        Err(poisoned) => poisoned.into_inner(),
                    };

                    l1_entries.remove(&host_clone);
                });
            }
        }
    }

    /// Clear the entire cache (both L1 and L2)
    pub fn clear(&self) -> Result<(), RedisError> {
        // Clear L1
        {
            let mut l1_entries = match self.l1_entries.write() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
            l1_entries.clear();

            let mut lru_queue = match self.lru_queue.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
            lru_queue.clear();
        }

        // Clear L2 (Redis)
        let mut con = self.get_connection()?;
        let _: () = redis::cmd("FLUSHDB").query(&mut con)?;

        Ok(())
    }
}

// Implement Clone for HttpCache to allow for thread-safe cloning
impl Clone for HttpCache {
    fn clone(&self) -> Self {
        HttpCache {
            l1_entries: self.l1_entries.clone(),
            hit_counters: self.hit_counters.clone(),
            db_client: self.db_client.clone(), // Clones only the client handle, not the connection
            config: CacheConfig {
                l1_max_size: self.config.l1_max_size,
                l1_default_ttl: self.config.l1_default_ttl,
                promotion_threshold: self.config.promotion_threshold,
                redis_url: self.config.redis_url.clone(),
            },
            lru_queue: self.lru_queue.clone(),
        }
    }
}
