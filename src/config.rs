#[derive(Clone)]
pub struct Config {
    pub ingest_addr: String,
    pub query_addr: String,
    pub rustikv_addr: String,
    pub ttl_secs: u32,
    pub batch_max_lines: usize,
    pub flush_ms: u64,
    pub max_buckets: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ingest_addr: "0.0.0.0:2003".into(),
            query_addr: "0.0.0.0:8080".into(),
            rustikv_addr: "127.0.0.1:6666".into(),
            ttl_secs: 86_400,
            batch_max_lines: 200,
            flush_ms: 1000,
            max_buckets: 2000,
        }
    }
}

impl Config {
    pub fn from_args(args: impl Iterator<Item = String>) -> Self {
        let mut c = Config::default();
        let mut it = args;
        while let Some(arg) = it.next() {
            match arg.as_str() {
                "--ingest" => {
                    if let Some(v) = it.next() {
                        c.ingest_addr = v;
                    }
                }
                "--query" => {
                    if let Some(v) = it.next() {
                        c.query_addr = v;
                    }
                }
                "--rustikv" => {
                    if let Some(v) = it.next() {
                        c.rustikv_addr = v;
                    }
                }
                "--ttl" => {
                    if let Some(v) = it.next() {
                        c.ttl_secs = v.parse().expect("--ttl seconds");
                    }
                }
                "--batch" => {
                    if let Some(v) = it.next() {
                        c.batch_max_lines = v.parse().expect("--batch lines");
                    }
                }
                "--flush-ms" => {
                    if let Some(v) = it.next() {
                        c.flush_ms = v.parse().expect("--flush-ms");
                    }
                }
                "--max-buckets" => {
                    if let Some(v) = it.next() {
                        c.max_buckets = v.parse().expect("--max-buckets");
                    }
                }
                other => eprintln!("ignoring unknown arg: {other}"),
            }
        }
        c
    }
}
