# Roxy

## To start
```
sudo cargo build
sudo ./target/debug/roxy
Safari -> Settings -> Advanced -> Change Settings -> Web Proxy (HTTP)/S
- host: 127.0.0.1
- port: 6505
```
Note that you have to turn off the proxy when **roxy** is not running

folder structure
```
src/
│── main.rs                   # Entry point, starts the proxy
│── cli/
│   │── console.r             # For commands
│── proxy/
│   ├── listener.rs           # Listens for incoming connections
│   ├── handler.rs            # Handles HTTP and HTTPS requests
│   ├── http.rs               # Forwards HTTP requests to real servers
│   ├── https.rs              # Handles HTTPS CONNECT tunneling
│   │── cache.rs              # Handles cache
│── utils/
│   ├── parsing.rs            # Parses HTTP requests, extracts hosts
│   │── host_filtering.rs     # Handles blacklisting of webpages
│   │── responses.rs          # Provides several predefined responses (e.g 403)
│   ├── logging.rs            # Handles logging and debugging
```
