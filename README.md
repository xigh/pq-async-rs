# 🦀 Rust Priority Queue

This repository accompanies a YouTube series on building a **Priority Queue** in Rust — step by step — from a minimal implementation to a full-fledged scheduling system.

The goal of the project is to explore:
- how to design robust queue data structures in Rust,
- how to evolve them into a fair, concurrent, asynchronous, and persistent scheduler,
- and how to do it in a clean, test-driven way.

---

## 🎥 Episodes

| Episode | Title | Branch | Tag | Video Link |
|----------|--------|---------|------|------|
| 01 | Rudimentary Implementation | `ep/01-rudimentary` | `ep01-v1.0` | https://youtu.be/dSCbdGIGKmo |
| 02 | Fair Scheduling Between Entities | `ep/02-fairness` | `ep02-v1.0` | https://youtu.be/rc7MkIPeWno |
| 03 | Workspaces, Threads & Shared Ownership (`Arc` & `Mutex`) | `ep/03-threadsafe` | `ep03-v1.0` | https://youtu.be/bguBATnHCcE |
| 04 | Blocking Queue & Controlled Shutdown (`Condvar` & `Result`) | `ep/04-blocking` | `ep04-v1.0` | https://youtu.be/jmXmDtdUFbg |

> Each branch corresponds to the **exact code version** used in the associated video.
> The `main` branch always contains the latest version of the project.

## Playlist

https://youtu.be/watch?v=dSCbdGIGKmo&list=PLMVdP7LZDmhgD3-RpxLsBX8Q9g7gXo_To&index=1

---

## 🧱 Current Architecture

Starting from episode 3, the project is organized as a **Cargo workspace** with multiple crates:

```

pq-async-rs/
├── Cargo.toml          # Workspace definition
└── crates/
├── pq-core         # Core types, errors, and traits
├── pq-fair         # Fairness logic (entity balancing)
├── pq-sync         # Thread-safe wrapper using Arc<Mutex<_>>
└── pq-examples     # Example binaries and test harnesses

````

Each crate focuses on a specific responsibility, allowing the queue to evolve cleanly toward async and distributed versions later.

---

## 🎯 Roadmap

| Stage | Focus | Description |
|--------|--------|-------------|
| **01** | Rudimentary Implementation | Basic `PriorityQueue` structure with enqueue/dequeue methods and simple unit tests. |
| **02** | Fair Scheduling | Implement **fairness mechanisms** to prevent starvation and ensure all entities get processing time. |
| **03** | Thread Safety | Introduce **multi-threading** with `Arc` and `Mutex` to make the queue safe across threads. |
| **04** | Async Runtime | Transition to an **asynchronous system** using Rust’s async/await model (`Tokio` or `async-std`). |
| **05** | Job Encapsulation | Wrap payloads inside a `Job` structure, adding job IDs, timestamps, and metadata. |
| **06** | Watchdog & Deadlines | Introduce a **watchdog** to re-schedule jobs on timeouts or failures. |
| **07** | Persistence | Add optional **durability** through file-backed or WAL-style persistence. |
| **08** | Authentication | Secure producer/consumer communication with access control. |
| **09** | Monitoring | Integrate **metrics collection** and a **Grafana dashboard** for insights. |

---

## 🧭 Branch & Version Strategy

| Branch | Purpose |
|---------|----------|
| `main` | Always points to the latest working version. |
| `ep/NN-<topic>` | Source branch for each episode or major milestone. |
| `tags/epNN-vX.Y` | Immutable tag matching the exact code version from each video. |

---

## 🧠 About the Project

This series is not about using libraries — it’s about understanding how they could be built.
We start simple, close to `std`, and progressively introduce concurrency, async behavior, and fairness.

The end goal is to create something that behaves like a lightweight **task broker**, but entirely written and reasoned from first principles.

---

## 🧩 Build and Run

```bash
# Clone the repository
git clone https://github.com/xigh/pq-async-rs
cd pq-async-rs

# Build all crates in the workspace
cargo build --workspace

# Run tests across all crates
cargo test --workspace

# Run an example
cargo run -p pq-examples
````

---

## ⚙️ Benchmarks

A dedicated benchmark crate (`pq-bench`) is included to compare the raw performance of the different queue implementations:

- `SyncPriorityQueue` (this project)
- `Crossbeam` (MPMC)
- `std::sync::mpsc`

The goal is not micro-optimization, but understanding **where the performance gap matters** — and where it doesn’t.

### Example results (Mac Studio M2 Max, release build)

| Impl | Producers | Consumers | Capacity | Work (ns) | p50 (ns) | Throughput (msg/s) |
|------|------------|------------|-----------|------------|-----------|--------------------|
| `xbeam` | 1 | 1 | 1 | 0 | 291 | 5,282,806 |
| `syncpq` | 1 | 1 | 1 | 0 | 8,875 | 163,018 |
| `xbeam` | 1 | 1 | 1 | 10,000 | 16,625 | 89,176 |
| `syncpq` | 1 | 1 | 1 | 10,000 | 16,417 | 88,188 |

> 🧩 In pure handoff (capacity = 1, no work), Crossbeam transfers a message in ~0.3 µs vs ~8.9 µs for `SyncPriorityQueue` → **≈ 8 µs fixed overhead**.
> Once you add **10 µs of processing per message**, both implementations are nearly identical (<1% difference).
> From **100 µs and above**, the performance gap becomes *insignificant*.

This confirms the design trade-off:
`SyncPriorityQueue` introduces minimal scheduling overhead, which is **negligible as soon as any real work is performed**.

### 🧪 To reproduce

```bash
cd crates/pq-bench
bash ../../benchs.sh
````

The script runs a full matrix of configurations (producers, consumers, capacity, and artificial work time), and outputs CSV-formatted results.

---

## 🔗 Resources Mentioned

* [std::thread](https://doc.rust-lang.org/std/thread/)
* [std::sync::Arc](https://doc.rust-lang.org/std/sync/struct.Arc.html)
* [std::sync::Mutex](https://doc.rust-lang.org/std/sync/struct.Mutex.html)
* [std::sync::Condvar](https://doc.rust-lang.org/std/sync/struct.Condvar.html)
* [std::result::Result](https://doc.rust-lang.org/std/result/enum.Result.html)
* [std::sync::PoisonError](https://doc.rust-lang.org/std/sync/struct.PoisonError.html)
* [Cargo Workspaces](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)
