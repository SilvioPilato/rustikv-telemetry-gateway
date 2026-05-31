pub struct Point {
    pub time: i64,
    pub value: f64,
}

/// Serialize points as a JSON array of `{"time":<unix_ms>,"value":<f64>}`.
/// Grafana's Infinity datasource column type `timestamp_epoch` expects milliseconds.
pub fn points_to_json(points: &[Point]) -> String {
    let mut out = String::from("[");
    for (i, p) in points.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        // `{}` on f64 prints 13.0 as "13"; that is valid JSON and fine for Grafana.
        out.push_str(&format!(r#"{{"time":{},"value":{}}}"#, p.time, p.value));
    }
    out.push(']');
    out
}
