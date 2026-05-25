use rustikv_telemetry_gateway::graphite::{Sample, parse_line};

#[test]
fn parses_valid_line() {
    let s = parse_line("pi.cpu.user 12.5 1748169600").unwrap();
    assert_eq!(
        s,
        Sample {
            metric: "pi.cpu.user".to_string(),
            value: 12.5,
            ts: 1_748_169_600
        }
    );
}

#[test]
fn accepts_integer_and_negative_values() {
    assert_eq!(parse_line("m 42 100").unwrap().value, 42.0);
    assert_eq!(parse_line("m -3.5 100").unwrap().value, -3.5);
}

#[test]
fn rejects_malformed_lines() {
    assert!(parse_line("too few").is_err());
    assert!(parse_line("m notanumber 100").is_err());
    assert!(parse_line("m 1.0 notatimestamp").is_err());
    assert!(parse_line("").is_err());
}
