pub mod cli {
    pub mod console;
}

pub mod proxy {
    pub mod listener;
    mod handler;
    mod forwarder;
    mod tunnel;
}

pub mod utils{
    pub mod logging;
    pub mod parsing;
    pub mod host_filtering;
    pub mod responses;
}

pub mod client {
    pub mod simple_client;
}
