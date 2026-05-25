use std::io::{Read, Write};
use std::net::TcpStream;

/// End-to-end roundtrip: feed Graphite lines into the gateway's ingest port,
/// then query them back through the gateway's HTTP port and assert the
/// server-side `avg` aggregation is correct.
///
/// Ignored by default — it requires a live rustikv LSM server plus a running
/// gateway. See the README "End-to-end verification" section to bring the stack
/// up, then run: `cargo test --test e2e -- --ignored`.
#[test]
#[ignore = "requires a live rustikv LSM server and the gateway running; see README"]
fn ingest_then_query_roundtrip() {
    // Assumes the gateway is reachable: ingest on 127.0.0.1:2003, query on 127.0.0.1:8080.
    let mut ingest = TcpStream::connect("127.0.0.1:2003").unwrap();
    for (i, ts) in [1000, 1060, 1120].iter().enumerate() {
        writeln!(ingest, "test.metric {}.0 {}", (i + 1) * 10, ts).unwrap();
    }
    drop(ingest); // disconnect triggers the final flush
    std::thread::sleep(std::time::Duration::from_millis(500));

    let mut q = TcpStream::connect("127.0.0.1:8080").unwrap();
    write!(
        q,
        "GET /query?metric=test.metric&from=900&to=1200&agg=avg&step=300 HTTP/1.1\r\n\r\n"
    )
    .unwrap();
    let mut resp = String::new();
    q.read_to_string(&mut resp).unwrap();
    let body = resp.split("\r\n\r\n").nth(1).unwrap();
    // avg of 10, 20, 30 = 20
    assert!(body.contains(r#""value":20"#), "got: {body}");
}
