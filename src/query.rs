use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

use rustikv::bffp::{Command, ResponseStatus};

use crate::config::Config;
use crate::http::parse_request_target;
use crate::json::{Point, points_to_json};
use crate::rustikv_client::Client;
use crate::schema::to_key;

/// Half-open `[from, to)` buckets of `step` seconds; last bucket clamps to `to`.
pub fn bucket_ranges(from: i64, to: i64, step: i64) -> Vec<(i64, i64)> {
    let mut out = Vec::new();
    let mut start = from;
    while start < to {
        let end = (start + step).min(to);
        out.push((start, end));
        start = end;
    }
    out
}

pub fn serve(config: Config) -> std::io::Result<()> {
    let listener = TcpListener::bind(&config.query_addr)?;
    eprintln!("query listening on {}", config.query_addr);
    let mut client = Client::connect(&config.rustikv_addr)?;
    for conn in listener.incoming().flatten() {
        if let Err(e) = handle(conn, &config, &mut client) {
            eprintln!("query conn error: {e}");
            let _ = client.reconnect();
        }
    }
    Ok(())
}

fn handle(mut conn: TcpStream, config: &Config, client: &mut Client) -> std::io::Result<()> {
    // Read just the request line (first line); we ignore headers/body for GET.
    let mut reader = BufReader::new(conn.try_clone()?);
    let mut line = String::new();
    reader.read_line(&mut line)?;

    let resp = match build_response(line.trim_end(), config, client) {
        Ok(json) => http_ok(&json),
        Err((code, msg)) => http_err(code, &msg),
    };
    conn.write_all(resp.as_bytes())
}

fn build_response(
    req_line: &str,
    config: &Config,
    client: &mut Client,
) -> Result<String, (u16, String)> {
    let target = parse_request_target(req_line).ok_or((400, "bad request line".to_string()))?;
    if target.path == "/health" {
        return Ok("ok".to_string());
    }
    if target.path != "/query" {
        return Err((404, "not found".to_string()));
    }
    let p = &target.params;
    let metric = p
        .get("metric")
        .ok_or((400, "metric required".to_string()))?;
    let from: i64 = p
        .get("from")
        .ok_or((400, "from required".to_string()))?
        .parse()
        .map_err(|_| (400u16, "bad from".to_string()))?;
    let to: i64 = p
        .get("to")
        .ok_or((400, "to required".to_string()))?
        .parse()
        .map_err(|_| (400u16, "bad to".to_string()))?;
    let agg = p.get("agg").unwrap_or("raw");

    if agg == "raw" {
        let points = query_raw(client, metric, from, to).map_err(|e| (502u16, e))?;
        return Ok(points_to_json(&points));
    }

    let step: i64 = p
        .get("step")
        .ok_or((400, "step required for aggregation".to_string()))?
        .parse()
        .map_err(|_| (400u16, "bad step".to_string()))?;
    if step <= 0 {
        return Err((400, "step must be positive".to_string()));
    }
    let buckets = bucket_ranges(from, to, step);
    if buckets.len() > config.max_buckets {
        return Err((
            400,
            format!(
                "too many buckets ({} > {})",
                buckets.len(),
                config.max_buckets
            ),
        ));
    }
    let mut points = Vec::new();
    for (b_from, b_to) in buckets {
        if let Some(v) = query_agg(client, agg, metric, b_from, b_to).map_err(|e| (502u16, e))? {
            points.push(Point {
                time_ms: b_from * 1000,
                value: v,
            });
        }
    }
    Ok(points_to_json(&points))
}

/// agg=raw: one RANGE over [from, to]; response is a FLAT list key,value,key,value...
fn query_raw(client: &mut Client, metric: &str, from: i64, to: i64) -> Result<Vec<Point>, String> {
    let resp = client
        .send(Command::Range(to_key(metric, from), to_key(metric, to)))
        .map_err(|e| e.to_string())?;
    let mut points = Vec::new();
    for pair in resp.payload.chunks_exact(2) {
        if let (Some((_, ts)), Ok(v)) = (
            crate::schema::parse_point_key(&pair[0]),
            pair[1].parse::<f64>(),
        ) {
            points.push(Point {
                time_ms: ts * 1000,
                value: v,
            });
        }
    }
    Ok(points)
}

/// agg=avg|min|max|sum over one bucket via the matching *Range op.
/// Returns None when the bucket is empty (rustikv replies NotFound / "Not found").
fn query_agg(
    client: &mut Client,
    agg: &str,
    metric: &str,
    from: i64,
    to: i64,
) -> Result<Option<f64>, String> {
    let (lo, hi) = (to_key(metric, from), to_key(metric, to));
    let cmd = match agg {
        "avg" => Command::AvgRange(lo, hi),
        "min" => Command::MinRange(lo, hi),
        "max" => Command::MaxRange(lo, hi),
        "sum" => Command::SumRange(lo, hi),
        other => return Err(format!("unknown agg: {other}")),
    };
    let resp = client.send(cmd).map_err(|e| e.to_string())?;
    match resp.status {
        ResponseStatus::Ok => Ok(resp.payload.first().and_then(|s| s.parse::<f64>().ok())),
        ResponseStatus::NotFound => Ok(None),
        ResponseStatus::Error => Err(resp.payload.join("; ")),
        ResponseStatus::Noop => Ok(None),
    }
}

fn http_ok(body: &str) -> String {
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    )
}

fn http_err(code: u16, msg: &str) -> String {
    let body = format!(r#"{{"error":"{msg}"}}"#);
    format!(
        "HTTP/1.1 {code} ERR\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    )
}
