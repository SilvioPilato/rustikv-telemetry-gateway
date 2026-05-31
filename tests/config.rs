use rustikv_telemetry_gateway::config::Config;

#[test]
fn defaults_are_sane() {
    let c = Config::default();
    assert_eq!(c.ingest_addr, "0.0.0.0:2003");
    assert_eq!(c.query_addr, "0.0.0.0:8080");
    assert_eq!(c.rustikv_addr, "127.0.0.1:6666");
    assert_eq!(c.ttl_secs, 86_400);
    assert!(c.batch_max_lines > 0 && c.flush_ms > 0 && c.max_buckets > 0);
    assert!(c.collection.is_none());
}

#[test]
fn parses_overrides_from_args() {
    let c = Config::from_args(
        ["--rustikv", "10.0.0.5:6666", "--ttl", "3600"]
            .iter()
            .map(|s| s.to_string()),
    );
    assert_eq!(c.rustikv_addr, "10.0.0.5:6666");
    assert_eq!(c.ttl_secs, 3600);
    assert!(c.collection.is_none());
}

#[test]
fn parses_collection_flag() {
    let c = Config::from_args(
        ["--collection", "metrics"].iter().map(|s| s.to_string()),
    );
    assert_eq!(c.collection.as_deref(), Some("metrics"));
}
