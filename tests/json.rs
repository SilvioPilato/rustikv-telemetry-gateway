use rustikv_telemetry_gateway::json::{Point, points_to_json};

#[test]
fn serializes_points_array() {
    let pts = vec![
        Point {
            time_ms: 100_000,
            value: 12.5,
        },
        Point {
            time_ms: 160_000,
            value: 13.0,
        },
    ];
    assert_eq!(
        points_to_json(&pts),
        r#"[{"time":100000,"value":12.5},{"time":160000,"value":13}]"#
    );
}

#[test]
fn serializes_empty_array() {
    assert_eq!(points_to_json(&[]), "[]");
}
