/// Fixed width for the zero-padded epoch-seconds suffix. 13 digits covers
/// epoch seconds well beyond the year 2286, keeping lexicographic order == time order.
pub const TS_WIDTH: usize = 13;

/// Build a rustikv key: `<metric>:<zero-padded-epoch-seconds>`.
pub fn to_key(metric: &str, ts_secs: i64) -> String {
    format!("{metric}:{ts_secs:0width$}", width = TS_WIDTH)
}

/// Split a point key back into (metric, ts_secs). Returns None if malformed.
/// The timestamp is the final colon-delimited segment.
pub fn parse_point_key(key: &str) -> Option<(String, i64)> {
    let idx = key.rfind(':')?;
    let (metric, ts) = (&key[..idx], &key[idx + 1..]);
    if metric.is_empty() {
        return None;
    }
    let ts_secs: i64 = ts.parse().ok()?;
    Some((metric.to_string(), ts_secs))
}
