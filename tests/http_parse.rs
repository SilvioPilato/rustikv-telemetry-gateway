use rustikv_telemetry_gateway::http::parse_request_target;

#[test]
fn parses_path_and_query_params() {
    let t = parse_request_target(
        "GET /query?metric=pi.cpu.user&from=100&to=200&agg=avg&step=60 HTTP/1.1",
    )
    .unwrap();
    assert_eq!(t.path, "/query");
    assert_eq!(t.params.get("metric"), Some("pi.cpu.user"));
    assert_eq!(t.params.get("from"), Some("100"));
    assert_eq!(t.params.get("agg"), Some("avg"));
    assert_eq!(t.params.get("step"), Some("60"));
}

#[test]
fn parses_path_without_query() {
    let t = parse_request_target("GET /health HTTP/1.1").unwrap();
    assert_eq!(t.path, "/health");
    assert_eq!(t.params.get("metric"), None);
}

#[test]
fn rejects_non_get_and_malformed() {
    assert!(parse_request_target("POST /query HTTP/1.1").is_none());
    assert!(parse_request_target("garbage").is_none());
}
