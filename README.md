# Roxy

## To start
```
sudo cargo build
sudo ./target/debug/roxy
Safari -> Settings -> Advanced -> Change Settings -> Web Proxy (HTTP)
- host: 127.0.0.1
- port: 80
```
Note that you have to turn off the proxy when **roxy** is not running

folder structure
```
src/
│── main.rs                   # Entry point, starts the proxy
│── proxy/
│   ├── mod.rs                # Proxy module
│   ├── listener.rs           # Listens for incoming connections
│   ├── handler.rs            # Handles HTTP and HTTPS requests
│   ├── forwarder.rs          # Forwards HTTP requests to real servers
│   ├── tunnel.rs             # Handles HTTPS CONNECT tunneling
│── utils/
│   ├── mod.rs                # Utility module
│   ├── parsing.rs            # Parses HTTP requests, extracts hosts
│   ├── logging.rs            # Handles logging and debugging
```
