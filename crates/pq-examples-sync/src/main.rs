use std::thread::spawn;

use pq_sync::SyncPriorityQueue;

fn main() {
    let pq = SyncPriorityQueue::new(3);

    let pq_clone = pq.clone();
    let jh = spawn(move || {
        let ret = pq_clone.enqueue(1, "A".to_string(), "A0".to_string());
        assert!(ret.is_ok());
    });

    jh.join().expect("join failed");

    let ret = pq.try_dequeue();
    assert!(ret.is_ok());
    let ret = ret.unwrap();
    assert!(ret.is_some());

    println!("{:?}", ret);
}
