# 🦀 Rust Priority Queue

This repository accompanies a YouTube series on building a **Priority Queue** in Rust — step by step — from a minimal implementation to a full-fledged scheduling system.

The goal of the project is to explore:
- how to design robust queue data structures in Rust,
- how to evolve them into a fair, asynchronous, and persistent scheduler,
- and how to do it in a clean, test-driven way.

---

## 🎥 Episodes

| Episode | Title | Branch | Tag | Video Link |
|----------|--------|---------|------|------|
| 01 | Rudimentary Implementation | `ep/01-rudimentary` | `ep01-v1.0` | https://youtu.be/dSCbdGIGKmo |
| 02 | Fair Scheduling Between Entities | `ep/02-fairness` | `ep02-v1.0` | https://youtu.be/rc7MkIPeWno |

> Each branch corresponds to the **exact code version** used in the associated video.
> The `main` branch always contains the latest version of the project.

---

## 🎯 Roadmap

| Stage | Focus | Description |
|--------|--------|-------------|
| **01** | Rudimentary Implementation | Basic `PriorityQueue` structure with enqueue/dequeue methods and simple unit tests. |
| **02** | Entities | Introduce **entities** representing producers and consumers. Each can have its own constraints and identifiers. |
| **03** | Job Encapsulation | Wrap payloads inside a `Job` structure, adding job IDs, timestamps, and metadata. |
| **04** | Fair Scheduling | Implement **fairness mechanisms** to prevent starvation and ensure that all entities get processing time according to their share. |
| **05** | Async Runtime | Transition the queue to an **asynchronous system** using Rust’s async/await model (likely with `Tokio`). Handle concurrent producers and consumers safely. |
| **06** | Watchdog & Deadlines | Introduce a **watchdog** mechanism to re-schedule jobs if they miss deadlines or fail to send heartbeats in time. |
| **07** | Persistence | Add optional **durability** through file-backed or WAL-style persistence, enabling recovery on restart. |
| **08** | Authentication | Secure communication between producers and consumers with authentication and access control. |
| **09** | Monitoring | Integrate **metrics collection** and build a **Grafana dashboard** to monitor queue states and job throughput. |

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
We’ll start simple, stay close to the `std` library, and progressively introduce asynchronous patterns (`Tokio`) and fairness mechanisms.

The end goal is to create something that behaves like a lightweight **task broker**, but entirely written and reasoned from first principles.

---

## 🧩 Build and Run

```bash
# Clone the repository
git clone https://github.com/xigh/pq-async-rs
cd pq-async-rs

# Run tests
cargo test

# Run examples (if/when available)
cargo run --example basic
