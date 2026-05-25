use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use std::time::{Duration, Instant};

use rustikv::bffp::Command;

use crate::config::Config;
use crate::graphite::parse_line;
use crate::rustikv_client::Client;
use crate::schema::to_key;

/// Accept Graphite-plaintext connections and forward batched samples to rustikv.
/// Single inbound connection at a time (sufficient for one collector); serve()
/// loops accepting connections sequentially.
pub fn serve(config: Config) -> std::io::Result<()> {
    let listener = TcpListener::bind(&config.ingest_addr)?;
    eprintln!("ingest listening on {}", config.ingest_addr);
    let mut client = Client::connect(&config.rustikv_addr)?;

    for conn in listener.incoming() {
        let conn = match conn {
            Ok(c) => c,
            Err(e) => {
                eprintln!("accept error: {e}");
                continue;
            }
        };
        if let Err(e) = handle_conn(conn, &config, &mut client) {
            eprintln!("connection ended: {e}");
        }
    }
    Ok(())
}

fn handle_conn(
    conn: std::net::TcpStream,
    config: &Config,
    client: &mut Client,
) -> std::io::Result<()> {
    let reader = BufReader::new(conn);
    let mut batch: Vec<(String, String, Option<u32>)> = Vec::new();
    let mut last_flush = Instant::now();
    let flush_after = Duration::from_millis(config.flush_ms);

    for line in reader.lines() {
        let line = line?;
        match parse_line(&line) {
            Ok(s) => batch.push((
                to_key(&s.metric, s.ts),
                s.value.to_string(),
                Some(config.ttl_secs),
            )),
            Err(e) => eprintln!("skip line {line:?}: {}", e.0),
        }
        if batch.len() >= config.batch_max_lines || last_flush.elapsed() >= flush_after {
            flush(client, &mut batch);
            last_flush = Instant::now();
        }
    }
    flush(client, &mut batch); // flush remaining on disconnect
    Ok(())
}

fn flush(client: &mut Client, batch: &mut Vec<(String, String, Option<u32>)>) {
    if batch.is_empty() {
        return;
    }
    let items = std::mem::take(batch);
    let n = items.len();
    if let Err(e) = client.send(Command::Mset(items)) {
        eprintln!("mset of {n} failed: {e}; reconnecting");
        let _ = client.reconnect();
    }
}
