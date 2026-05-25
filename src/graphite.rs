#[derive(Debug, PartialEq)]
pub struct Sample {
    pub metric: String,
    pub value: f64,
    pub ts: i64,
}

#[derive(Debug, PartialEq)]
pub struct ParseError(pub String);

/// Parse one Graphite plaintext line: `metric.path value timestamp`.
pub fn parse_line(line: &str) -> Result<Sample, ParseError> {
    let mut parts = line.split_whitespace();
    let metric = parts
        .next()
        .ok_or_else(|| ParseError("missing metric".into()))?;
    let value_s = parts
        .next()
        .ok_or_else(|| ParseError("missing value".into()))?;
    let ts_s = parts
        .next()
        .ok_or_else(|| ParseError("missing timestamp".into()))?;
    if parts.next().is_some() {
        return Err(ParseError("too many fields".into()));
    }
    let value: f64 = value_s
        .parse()
        .map_err(|_| ParseError(format!("bad value: {value_s}")))?;
    let ts: i64 = ts_s
        .parse()
        .map_err(|_| ParseError(format!("bad timestamp: {ts_s}")))?;
    Ok(Sample {
        metric: metric.to_string(),
        value,
        ts,
    })
}
