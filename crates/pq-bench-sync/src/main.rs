//! Micro-benchmark: raw enqueue→dequeue latency & throughput,
//! with clean shutdown that is EXCLUDED from the measurement window.
//!
//! Measurement window:
//! - Start when producers are released by a barrier.
//! - Stop after we've collected exactly N Data latencies on the aggregator.
//! - Then, we shutdown (poison pills or queue shutdown) OUTSIDE the window.
//!
//! Implementations compared:
//! - syncpq : your SyncPriorityQueue wrapped with a bounded layer (1 priority, 1 entity).
//! - xbeam  : crossbeam::bounded MPMC.
//! - mpsc   : std::sync::mpsc::sync_channel bounded (single real consumer).
//!
//! Output CSV:
//! impl,producers,consumers,n_items,capacity,p50_ns,p95_ns,p99_ns,throughput_items_per_s
//!
//! Run (build release to reduce noise):
//!   cargo build --release
//!   target/release/pq-bench --implm syncpq --producers 4 --consumers 4 --n-items 500000 --capacity 1024
//!   target/release/pq-bench --implm xbeam  --producers 4 --consumers 4 --n-items 500000 --capacity 1024
//!   target/release/pq-bench --implm mpsc  --producers 4 --consumers 4 --n-items 500000 --capacity 1024

use anyhow::Result;
use clap::Parser;
use crossbeam_channel as xbeam;
use pq_sync::SyncPriorityQueue;
use std::{
    fmt::Debug,
    sync::{Arc, Barrier, Condvar, Mutex, mpsc as stdmpsc},
    thread,
    time::Instant,
};

#[derive(Parser, Debug, Clone)]
struct Args {
    /// "syncpq" | "xbeam" | "mpsc"
    #[arg(long, default_value = "syncpq")]
    implm: String,

    /// number of producers
    #[arg(long, default_value_t = 4)]
    producers: usize,

    /// number of consumers
    #[arg(long, default_value_t = 4)]
    consumers: usize,

    /// total items (Data messages) to measure
    #[arg(long, default_value_t = 200_000)]
    n_items: usize,

    /// queue depth / capacity
    #[arg(long, default_value_t = 1024)]
    capacity: usize,

    /// shutdown mode for syncpq: "immediate" | "graceful"
    #[arg(long, default_value = "immediate")]
    shutdown: String,

    /// CPU work per item in nanoseconds (busy-wait), executed by consumers after each message
    #[arg(long, default_value_t = 0u64)]
    work_ns: u64,
}

#[derive(Clone)]
struct Stamp {
    t: Instant,
}

/// Unified message so we can send poison pills without polluting metrics.
#[derive(Clone)]
enum Msg {
    Data(Stamp),
    Stop,
}

// ---------- Adapter trait: enqueue/dequeue + optional shutdown (excluded from timing) ----

trait QueueAdapter: Send + Sync + 'static {
    fn enqueue_data(&self, m: Msg);
    fn dequeue(&self) -> Msg; // blocking
    fn shutdown_immediate(&self) {} // default no-op
    fn shutdown_graceful(&self) {} // default no-op
}

// ------------------------ Crossbeam -----------------------------------------

struct XBeamAdapter {
    tx: xbeam::Sender<Msg>,
    rx: xbeam::Receiver<Msg>,
}
impl QueueAdapter for XBeamAdapter {
    fn enqueue_data(&self, m: Msg) {
        self.tx.send(m).unwrap();
    }
    fn dequeue(&self) -> Msg {
        self.rx.recv().unwrap()
    }
}

// ------------------------ std::mpsc -----------------------------------------

struct MpscAdapter {
    tx: stdmpsc::SyncSender<Msg>,
    // Receiver<T> is not Sync → protect it; this also matches single-consumer semantics.
    rx: Mutex<stdmpsc::Receiver<Msg>>,
}
impl QueueAdapter for MpscAdapter {
    fn enqueue_data(&self, m: Msg) {
        self.tx.send(m).unwrap();
    }
    fn dequeue(&self) -> Msg {
        let rx = self.rx.lock().unwrap();
        rx.recv().unwrap()
    }
}

// ------------------------ SyncPriorityQueue (bounded wrapper) ---------------
//
// SyncPriorityQueue itself is unbounded. To compare apples-to-apples against
// bounded channels, we add a tiny "capacity gate":
// - producers block in enqueue when the inflight count reaches 'cap'
// - consumers release a slot after dequeue(Data)

struct BoundedSyncPQAdapter {
    pq: SyncPriorityQueue<usize, Msg>,
    cap: usize,
    gate: Gate,
}

struct Gate {
    mu: Mutex<usize>, // inflight count (enqueued - dequeued), only for Data
    cv: Condvar,
}

impl BoundedSyncPQAdapter {
    fn new(cap: usize) -> Self {
        Self {
            pq: SyncPriorityQueue::<usize, Msg>::new(1),
            cap,
            gate: Gate {
                mu: Mutex::new(0),
                cv: Condvar::new(),
            },
        }
    }

    // Acquire one slot; block while capacity is full.
    fn acquire_slot(&self) {
        let mut n = self.gate.mu.lock().unwrap();
        while *n >= self.cap {
            n = self.gate.cv.wait(n).unwrap();
        }
        *n += 1;
    }

    // Release one slot and wake a waiting producer (if any).
    fn release_slot(&self) {
        let mut n = self.gate.mu.lock().unwrap();
        *n -= 1;
        self.gate.cv.notify_one();
    }
}

impl QueueAdapter for BoundedSyncPQAdapter {
    fn enqueue_data(&self, m: Msg) {
        if matches!(m, Msg::Data(_)) {
            self.acquire_slot();
        }
        // Single priority (0), single entity (0) for apples-to-apples micro-bench.
        self.pq.enqueue(0, 0, m).unwrap();
    }

    fn dequeue(&self) -> Msg {
        let msg = self.pq.dequeue().unwrap();
        if matches!(msg, Msg::Data(_)) {
            self.release_slot();
        }
        msg
    }

    fn shutdown_immediate(&self) {
        let _ = self.pq.shutdown_immediate();
    }
    fn shutdown_graceful(&self) {
        let _ = self.pq.shutdown_graceful();
    }
}

// ----------------------------------------------------------------------------

fn busy_work_ns(ns: u64) {
    if ns == 0 {
        return;
    }
    let start = std::time::Instant::now();
    // tight spin; enough for démonstration (évite sleep, granulosité trop grossière)
    while start.elapsed().as_nanos() < ns as u128 {
        std::hint::spin_loop();
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Build adapter
    let adapter: Box<dyn QueueAdapter> = match args.implm.as_str() {
        "xbeam" => {
            let (tx, rx) = xbeam::bounded::<Msg>(args.capacity);
            Box::new(XBeamAdapter { tx, rx })
        }
        "mpsc" => {
            let (tx, rx) = stdmpsc::sync_channel::<Msg>(args.capacity);
            Box::new(MpscAdapter {
                tx,
                rx: Mutex::new(rx),
            })
        }
        "syncpq" => Box::new(BoundedSyncPQAdapter::new(args.capacity)),
        other => {
            eprintln!("Unknown --implm={other}. Use 'syncpq' | 'xbeam' | 'mpsc'.");
            std::process::exit(2);
        }
    };
    let q = Arc::new(adapter);

    // Barrier to start all producers at once (stable contention).
    let start_barrier = Arc::new(Barrier::new(args.producers + 1));

    // Latency aggregator (nanoseconds)
    let (lat_tx, lat_rx) = xbeam::unbounded::<u64>();

    // Consumers: block, record Data latencies, exit upon receiving Stop.
    let mut consumer_handles = Vec::with_capacity(args.consumers);
    for _ in 0..args.consumers {
        let q = Arc::clone(&q);
        let lat_tx = lat_tx.clone();
        consumer_handles.push(thread::spawn(move || {
            while let Msg::Data(stamp) = q.dequeue() {
                let ns = stamp.t.elapsed().as_nanos() as u64;
                // If the aggregator is already complete, ignoring send errors is fine.
                let _ = lat_tx.send(ns);
                busy_work_ns(args.work_ns);
            }
        }));
    }

    // Split Data items across producers
    let base = args.n_items / args.producers;
    let extra = args.n_items % args.producers;

    // Producers
    let mut producers = Vec::with_capacity(args.producers);
    for pid in 0..args.producers {
        let q = Arc::clone(&q);
        let n = base + if pid < extra { 1 } else { 0 };
        let sb = Arc::clone(&start_barrier);
        producers.push(thread::spawn(move || {
            // Wait for synchronized start
            sb.wait();
            for _ in 0..n {
                q.enqueue_data(Msg::Data(Stamp { t: Instant::now() }));
            }
        }));
    }

    // Start the measurement window: when we release producers.
    let t0 = Instant::now();
    start_barrier.wait();

    // Collect exactly N Data latencies → end of measurement window.
    let mut lats = Vec::with_capacity(args.n_items);
    for _ in 0..args.n_items {
        let ns = lat_rx.recv().unwrap();
        lats.push(ns);
    }
    let elapsed = t0.elapsed();

    // After measurement: clean shutdown (excluded from metrics)
    // 1) Ask consumers to stop (poison pills): send exactly `consumers` Stop messages.
    for _ in 0..args.consumers {
        q.enqueue_data(Msg::Stop);
    }

    // 2) Join producers (they’re already done by design)
    for h in producers {
        h.join().unwrap();
    }

    // 3) Join consumers
    for h in consumer_handles {
        h.join().unwrap();
    }

    // 4) For SyncPriorityQueue, call the chosen shutdown after everything (no effect on metrics)
    if args.implm.as_str() == "syncpq" {
        match args.shutdown.as_str() {
            "immediate" => q.shutdown_immediate(),
            "graceful" => q.shutdown_graceful(),
            _ => {}
        }
    }

    // Compute metrics
    lats.sort_unstable();
    let p50 = percentile(&lats, 50.0);
    let p95 = percentile(&lats, 95.0);
    let p99 = percentile(&lats, 99.0);
    let tps = args.n_items as f64 / elapsed.as_secs_f64();

    println!(
        "impl,producers,consumers,n_items,capacity,p50_ns,p95_ns,p99_ns,throughput_items_per_s"
    );
    println!(
        "{},{},{},{},{},{},{},{},{}",
        args.implm,
        args.producers,
        args.consumers,
        args.n_items,
        args.capacity,
        p50 as u64,
        p95 as u64,
        p99 as u64,
        tps as u64
    );

    Ok(())
}

/// Percentile (nearest-rank) on sorted ns
fn percentile(sorted_ns: &[u64], p: f64) -> f64 {
    if sorted_ns.is_empty() {
        return 0.0;
    }
    let n = sorted_ns.len();
    let rank = ((p / 100.0) * (n as f64 - 1.0)).round() as usize;
    sorted_ns[rank] as f64
}
