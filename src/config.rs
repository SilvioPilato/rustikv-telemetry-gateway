#[derive(Clone)]
pub struct Config {
    pub ingest_addr: String,
    pub query_addr: String,
    pub rustikv_addr: String,
    /// Per-key TTL applied on every write. Ignored when `collection` is set
    /// (the collection's server-side default TTL is used instead).
    pub ttl_secs: u32,
    pub batch_max_lines: usize,
    pub flush_ms: u64,
    pub max_buckets: usize,
    /// If set, the gateway issues `USE <collection>` on connect and relies on
    /// the collection's default TTL rather than sending per-key TTLs.
    pub collection: Option<String>,
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
            collection: None,
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
                "--collection" => {
                    if let Some(v) = it.next() {
                        c.collection = Some(v);
                    }
                }
                other => eprintln!("ignoring unknown arg: {other}"),
            }
        }
        c
    }
}
