use std::collections::HashMap;

pub struct QueryParams(HashMap<String, String>);
impl QueryParams {
    pub fn get(&self, k: &str) -> Option<&str> {
        self.0.get(k).map(|s| s.as_str())
    }
}

pub struct RequestTarget {
    pub path: String,
    pub params: QueryParams,
}

/// Parse an HTTP request line of the form `GET /path?a=b&c=d HTTP/1.1`.
/// Returns None for non-GET or malformed lines. Values are NOT percent-decoded
/// (metric names and integer params used here don't require it).
pub fn parse_request_target(line: &str) -> Option<RequestTarget> {
    let mut parts = line.split_whitespace();
    if parts.next()? != "GET" {
        return None;
    }
    let target = parts.next()?;
    let (path, query) = match target.split_once('?') {
        Some((p, q)) => (p, q),
        None => (target, ""),
    };
    let mut params = HashMap::new();
    for pair in query.split('&').filter(|s| !s.is_empty()) {
        if let Some((k, v)) = pair.split_once('=') {
            params.insert(k.to_string(), v.to_string());
        }
    }
    Some(RequestTarget {
        path: path.to_string(),
        params: QueryParams(params),
    })
}
