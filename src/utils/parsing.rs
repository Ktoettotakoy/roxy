use std::collections::HashMap;

/// module used for functions that perform some kind of parsing


/// Extracts host and port (443 or 80)
/// # Parameters
/// * request - preferably str slice of HTTP request
///
/// - returns Option<String>
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


#[derive(Debug)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

#[derive(Debug)]
pub struct HttpResponse {
    pub version: String,
    pub status_code: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}



/// Takes the response and transforms it into a struct
pub fn parse_http_response(raw_response: &str) -> Option<HttpResponse> {
    let mut lines = raw_response.lines();


    let first_line = lines.next()?.trim();
    let mut parts = first_line.split_whitespace();


    let version = parts.next()?.to_string();
    let status_code = parts.next()?.parse::<u16>().ok()?;
    let status_text = parts.collect::<Vec<&str>>().join(" ");


    let mut headers = HashMap::new();
    let mut body = Vec::new();
    let mut is_body = false;

    for line in lines {
        if line.is_empty() {
            // if it hits an empty line before end of lines, then response has body
            is_body = true;
            continue;
        }

        if is_body {
            body.extend_from_slice(line.as_bytes());
        } else if let Some((key, value)) = line.split_once(": ") {
            headers.insert(key.to_string(), value.to_string());
        }
    }

    return Some(HttpResponse {
        version,
        status_code,
        status_text,
        headers,
        body: body,
    });
}


/// Takes the request and transforms it into a struct
pub fn parse_http_request(raw_request: &str) -> Option<HttpRequest> {
    let mut lines = raw_request.lines();


    let first_line = lines.next()?.trim();
    let mut parts = first_line.split_whitespace();


    let method = parts.next()?.to_string();
    let path = parts.next()?.to_string();
    let version = parts.next()?.to_string();


    let mut headers = HashMap::new();
    let mut body = Vec::new();
    let mut is_body = false;

    for line in lines {
        if line.is_empty() {
            // if it hits an empty line before end of lines, then request has body
            is_body = true;
            continue;
        }

        if is_body {
            body.extend_from_slice(line.as_bytes());
        } else if let Some((key, value)) = line.split_once(": ") {
            headers.insert(key.to_string(), value.to_string());
        }
    }

    return Some(HttpRequest {
        method,
        path,
        version,
        headers,
        body: body,
    });

}
