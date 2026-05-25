use std::thread;

use rustikv_telemetry_gateway::config::Config;
use rustikv_telemetry_gateway::{ingest, query};

fn main() -> std::io::Result<()> {
    let config = Config::from_args(std::env::args().skip(1));
    let ingest_cfg = config.clone();
    let ingest_handle = thread::spawn(move || {
        if let Err(e) = ingest::serve(ingest_cfg) {
            eprintln!("ingest fatal: {e}");
        }
    });
    let query_handle = thread::spawn(move || {
        if let Err(e) = query::serve(config) {
            eprintln!("query fatal: {e}");
        }
    });
    ingest_handle.join().unwrap();
    query_handle.join().unwrap();
    Ok(())
}
