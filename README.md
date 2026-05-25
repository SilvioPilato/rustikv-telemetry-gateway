# rustikv-telemetry-gateway

A small standalone gateway that bridges standard metrics tooling to
[rustikv](https://github.com/SilvioPilato/rustikv) — a didactic key-value store
with an LSM engine and server-side aggregation. rustikv speaks a custom binary
protocol (BFFP) that no collector or dashboard talks natively, so this gateway
translates in both directions:

- **Ingest:** accepts **Graphite plaintext** from any collector (collectd,
  Telegraf, statsd-graphite) and writes it into rustikv as batched `MSET`s with a
  TTL.
- **Serve:** exposes an HTTP **`/query`** endpoint returning JSON time series for
  Grafana's [Infinity datasource](https://grafana.com/grafana/plugins/yesoreyeram-infinity-datasource/),
  pushing time-window downsampling *server-side* into rustikv's aggregation ops.

```
Raspberry Pi                          gateway host                       rustikv host
┌────────────┐  Graphite plaintext  ┌─────────────────────┐  BFFP/TCP  ┌──────────┐
│ collectd / │  metric value ts\n   │ rustikv-telemetry-  │  MSET+TTL  │ rustikv  │
│ Telegraf   │ ───── TCP:2003 ────▶ │ gateway             │ ──:6666──▶ │  (LSM)   │
└────────────┘                      │                     │            └────┬─────┘
                                    │  HTTP /query  ◀─────┼── Grafana       │ RANGE /
┌────────────┐  HTTP/JSON           │  (Infinity DS)      │ ────────────────┘ AVG/MIN/MAX
│  Grafana   │ ◀─── :8080 ───────── │                     │
└────────────┘                      └─────────────────────┘
```

It reuses rustikv's `bffp` module (via a git dependency) for the wire protocol —
no protocol re-implementation, no drift. The HTTP/1.1 GET handling and JSON
output are hand-rolled, so the only dependency is the `rustikv` crate.

## Build

```sh
cargo build
```

This fetches `rustikv` from GitHub (`bffp` is public on its `main` branch).

## Run

```sh
cargo run -- [options]
```

| Flag | Description | Default |
|------|-------------|---------|
| `--ingest <addr>` | TCP address for Graphite plaintext ingest | `0.0.0.0:2003` |
| `--query <addr>` | HTTP address for the `/query` API | `0.0.0.0:8080` |
| `--rustikv <addr>` | rustikv server address | `127.0.0.1:6666` |
| `--ttl <seconds>` | TTL applied to every written sample (retention) | `86400` (24h) |
| `--batch <lines>` | Flush a batch after this many lines | `200` |
| `--flush-ms <ms>` | Flush a batch after this many milliseconds | `1000` |
| `--max-buckets <n>` | Reject `/query` requests exceeding this many aggregation buckets | `2000` |

rustikv **must run with the LSM engine** (`--engine lsm`) — `RANGE` and the
aggregation ops are LSM-only.

## Key schema

Each Graphite line `metric.path value timestamp` becomes a rustikv key/value:

```
key:   <metric.path>:<zero-padded-13-digit-epoch-seconds>     value: <numeric value>
e.g.   pi.cpu.user:0001748169600                               12.5
```

The zero-padded fixed-width timestamp makes rustikv's **lexicographic** `RANGE`
order match numeric time order, so window queries and aggregation work directly.

## Query API

```
GET /query?metric=<name>&from=<epoch>&to=<epoch>&agg=raw|avg|min|max|sum&step=<sec>
→  200  [ { "time": <epoch_ms>, "value": <number> }, ... ]
```

- `agg=raw` — one `RANGE` over `[from, to]`; returns every stored point.
- `agg=avg|min|max|sum` with `step` — splits the window into `step`-second
  buckets and issues one rustikv `AvgRange`/`MinRange`/`MaxRange`/`SumRange` per
  bucket, returning one rolled-up point per bucket. Downsampling happens
  server-side in rustikv.
- `GET /health` → `200 ok`.

Empty buckets are omitted from the series. Bad parameters return `400`; a rustikv
error returns `502`.

## Grafana

Install the **Infinity** datasource plugin. Use the panel in
[`examples/grafana-panel.json`](examples/grafana-panel.json) (replace
`GATEWAY_IP` and the datasource uid). It parses the JSON `time` (epoch ms) and
`value` columns into a time series, with `agg=avg&step=60` for 1-minute
server-side rollups across the dashboard time range.

## Raspberry Pi collector

See [`examples/collectd-graphite.conf`](examples/collectd-graphite.conf) for a
collectd `write_graphite` config that ships host metrics (CPU, memory, load,
disk, SoC temperature) to the gateway. Replace `GATEWAY_IP`.

## End-to-end verification

```sh
# Terminal 1 — rustikv (LSM engine)
cd ../kv-store && cargo run -- C:/Temp/gw-db --engine lsm

# Terminal 2 — the gateway
cd ../rustikv-telemetry-gateway && cargo run

# Terminal 3 — push a Graphite sample and query it back (PowerShell):
$c = New-Object Net.Sockets.TcpClient("127.0.0.1", 2003); $s = $c.GetStream()
$b = [Text.Encoding]::ASCII.GetBytes("demo.metric 42.0 1748169600`n")
$s.Write($b, 0, $b.Length); $c.Close()
curl "http://127.0.0.1:8080/query?metric=demo.metric&from=1748169000&to=1748170000&agg=avg&step=300"
# → [{"time":1748169000000,"value":42}]
```

There is also an automated end-to-end test (ignored by default, since it needs
the live stack above):

```sh
cargo test --test e2e -- --ignored
```

## Status

Experimental / didactic, like rustikv itself. Single inbound connection at a
time (sufficient for one collector); aggregation buckets use rustikv's inclusive
`*Range`, so a sample exactly on a bucket boundary can be counted in two adjacent
buckets — acceptable for this prototype.
