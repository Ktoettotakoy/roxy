use std::io::{Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

pub fn send_get_every_5_seconds_to_port(port: u16) {

    let host = "192.168.0.100";

    let mut stream = TcpStream::connect((host, port)).expect("Failed to connect");

    loop {
        println!("Sent request to {}", host);
        let request = "GET / HTTP/1.1\r\nHost: localhost\r\nConnection: keep-alive\r\n\r\n";
        stream.write_all(request.as_bytes()).expect("Failed to write");
        thread::sleep(Duration::from_secs(5));
    }

}
