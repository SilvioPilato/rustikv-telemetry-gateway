use rustikv_telemetry_gateway::schema::{TS_WIDTH, parse_point_key, to_key};

#[test]
fn to_key_zero_pads_timestamp_to_fixed_width() {
    assert_eq!(
        to_key("pi.cpu.user", 1_748_169_600),
        "pi.cpu.user:0001748169600"
    );
    assert_eq!(
        to_key("pi.cpu.user", 0).len(),
        "pi.cpu.user:".len() + TS_WIDTH
    );
}

#[test]
fn keys_sort_lexicographically_in_timestamp_order() {
    let earlier = to_key("m", 100);
    let later = to_key("m", 2000);
    assert!(
        earlier < later,
        "lexicographic order must match numeric time order"
    );
}

#[test]
fn parse_point_key_round_trips() {
    let k = to_key("pi.mem.used", 1_748_169_600);
    assert_eq!(
        parse_point_key(&k),
        Some(("pi.mem.used".to_string(), 1_748_169_600))
    );
}

#[test]
fn parse_point_key_rejects_malformed() {
    assert_eq!(parse_point_key("no-colon"), None);
    assert_eq!(parse_point_key("metric:notanumber"), None);
}
