pub struct Point {
    pub time_ms: i64,
    pub value: f64,
}

/// Serialize points as a JSON array of `{"time":<ms>,"value":<f64>}`.
/// Grafana's Infinity datasource maps this directly to a time series.
pub fn points_to_json(points: &[Point]) -> String {
    let mut out = String::from("[");
    for (i, p) in points.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        // `{}` on f64 prints 13.0 as "13"; that is valid JSON and fine for Grafana.
        out.push_str(&format!(r#"{{"time":{},"value":{}}}"#, p.time_ms, p.value));
    }
    out.push(']');
    out
}
