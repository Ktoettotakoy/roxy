/// module used for functions that perform some kind of parsing


/// parses http request to extract host from it
/// returns Options<String>
// pub fn extract_host(request: &str) -> Option<String> {
//     for line in request.lines() {
//         if line.to_lowercase().starts_with("host:") {
//             return Some(line[5..].trim().to_string());
//         }
//     }
//     None
// }

pub fn extract_host(request: &str) -> Option<String> {
    // For CONNECT requests
    if request.starts_with("CONNECT") {
        let lines: Vec<&str> = request.split("\r\n").collect();
        if let Some(first_line) = lines.first() {
            let parts: Vec<&str> = first_line.split_whitespace().collect();
            if parts.len() >= 2 {
                let host_port = parts[1];
                // Check if the host includes a port
                if !host_port.contains(":") {
                    // Add default HTTPS port if missing
                    return Some(format!("{}:443", host_port));
                }
                return Some(host_port.to_string());
            }
        }
    }
    // For HTTP requests
    else {
        // Look for "Host:" header
        for line in request.lines() {
            if line.to_lowercase().starts_with("host:") {
                let host = line[5..].trim();
                // For HTTP requests, add port 80 if not specified
                if !host.contains(":") {
                    return Some(format!("{}:80", host));
                }
                return Some(host.to_string());
            }
        }
    }
    None
}
