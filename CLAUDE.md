# iron-redis

A high-performance, asynchronous, in-memory key-value store written in Rust.

## Project Goals

- Redis-like semantics: an in-memory data store accessed over a network protocol.
- Low latency and high throughput are the primary success metrics. Every design
  decision should be weighed against its cost in allocations, copies, and lock
  contention.

## Memory Rules (strict)

- **Minimize allocations.** Do not allocate (`String`, `Vec`, `Box`, etc.) where a
  borrow will do. Prefer `&str` / `&[u8]` slices over owned `String` / `Vec<u8>`
  whenever the data does not need to outlive the buffer it came from.
- **Zero-copy parsing is mandatory for the protocol layer.** Command/frame parsing
  must operate on slices into the original input buffer (e.g. `&[u8]` windows,
  `bytes::Bytes` if reference-counted zero-copy buffers are needed across task
  boundaries). Do not parse by copying into intermediate `String`/`Vec<u8>` values
  and do not use `format!`/`to_string`/`to_owned`/`.clone()` in hot paths.
- Only allocate/own data when it must cross a lifetime boundary (e.g. stored into
  the keyspace, sent across an `await` point into another task, or returned to a
  caller that outlives the source buffer). When you do, justify it in a short
  comment if it's non-obvious why a borrow wasn't possible.
- Avoid unnecessary intermediate collections (`collect::<Vec<_>>()` followed by
  another pass, etc.). Prefer iterator chains that do one pass over the data.

## Concurrency Rules (strict)

- **Use `std::sync::Arc` and `std::sync::RwLock` (or `tokio::sync::RwLock` for
  lock-holding across `.await` points) to share and guard collections.** The
  keyspace and any other shared mutable state must be protected this way.
- Prefer many reads / few writes: reach for `RwLock` over `Mutex` when read
  access dominates, which is the expected access pattern for a KV store.
- **Do not introduce external unsafe/lock-free crates** (e.g. `crossbeam`,
  `dashmap`, `flurry`, `arc-swap`, etc.) as a substitute for `Arc`/`RwLock`.
  If a standard `Arc<RwLock<T>>` pattern is measured and proven insufficient,
  raise it for discussion before adding a dependency or writing `unsafe` code.
- **No `unsafe` blocks.** This project does not write unsafe Rust. If a
  performance problem seems to require it, solve it with better data
  structures, sharding of `RwLock`-guarded collections, or algorithmic changes
  first.
- Keep critical sections small: acquire the lock, do the minimal work needed,
  release it. Never hold a lock across an `.await` point unless using a
  lock type explicitly designed for that (e.g. `tokio::sync::RwLock`).

## Build & Test Workflow

Use standard cargo tooling only — no custom build scripts or task runners
unless explicitly introduced and documented here.

- Build: `cargo build`
- Build (release/optimized): `cargo build --release`
- Run: `cargo run`
- Test: `cargo test`
- Format: `cargo fmt`
- Lint: `cargo clippy --all-targets -- -D warnings`

Run `cargo fmt` and `cargo clippy` before considering any change complete.
All new functionality should be accompanied by `cargo test` coverage.
