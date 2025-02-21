/// module used for functions that perform some kind of parsing


/// parses http request to extract host from it
/// returns Options<String>
pub fn extract_host(request: &str) -> Option<String> {
    for line in request.lines() {
        if line.to_lowercase().starts_with("host:") {
            return Some(line[5..].trim().to_string());
        }
    }
    None
}
