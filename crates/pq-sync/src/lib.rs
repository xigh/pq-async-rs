use std::{
    hash::Hash,
    sync::{Arc, Condvar, Mutex},
    time::Duration,
};

use pq_core::{PriorityQueueError, Result};
use pq_fair::PriorityQueue;

struct State<E, T>
where
    E: Eq + Hash + Clone,
{
    pq: PriorityQueue<E, T>,
    closed: bool,
}

impl<E, T> State<E, T>
where
    E: Eq + Hash + Clone,
{
    fn new(n_prio: usize) -> Self {
        Self {
            pq: PriorityQueue::new(n_prio),
            closed: false,
        }
    }
}

struct Inner<E, T>
where
    E: Eq + Hash + Clone,
{
    state: Mutex<State<E, T>>,
    cv: Condvar,
}

impl<E, T> Inner<E, T>
where
    E: Eq + Hash + Clone,
{
    fn new(n_prio: usize) -> Self {
        Self {
            state: Mutex::new(State::new(n_prio)),
            cv: Condvar::new(),
        }
    }
}

#[derive(Clone)]
pub struct SyncPriorityQueue<E, T>
where
    E: Eq + Hash + Clone,
{
    inner: Arc<Inner<E, T>>,
}

impl<E, T> SyncPriorityQueue<E, T>
where
    E: Eq + Hash + Clone,
{
    /// Creates a new synchronized priority queue with a fixed number of priority levels.
    ///
    /// This constructor initializes the internal structure of the queue with `n_prio`
    /// priority levels, each maintaining its own fair sub-queue per entity.
    ///
    /// It is fully thread-safe and ready to be shared between producers and consumers
    /// via cloning (`Clone`), as the internal state is wrapped in [`std::sync::Arc`].
    ///
    /// # Arguments
    ///
    /// * `n_prio` — The number of priority levels in the queue (must be greater than 0).
    ///
    /// # Panics
    ///
    /// This function will **panic** if `n_prio` is zero, as at least one level is required.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pq_sync::SyncPriorityQueue;
    ///
    /// // Create a queue with three priority levels
    /// let pq = SyncPriorityQueue::<String, String>::new(3);
    ///
    /// // It can be safely cloned across threads
    /// let pq_clone = pq.clone();
    ///
    /// std::thread::spawn(move || {
    ///     pq_clone.enqueue(0, "client_A".to_string(), "task_1".to_string()).unwrap();
    /// });
    /// ```
    ///
    /// # See also
    /// * [`enqueue()`] — Add an item to the queue.
    /// * [`dequeue()`] — Remove an item, blocking if necessary.
    ///
    pub fn new(n_prio: usize) -> Self {
        assert!(n_prio > 0, "n_prio must be > 0");
        Self {
            inner: Arc::new(Inner::new(n_prio)),
        }
    }
}

/// ---
/// ## Queue Operations
///
/// Core blocking operations for producing and consuming items.
///
impl<E, T> SyncPriorityQueue<E, T>
where
    E: Eq + Hash + Clone,
{
    /// Enqueues a new item into the priority queue.
    ///
    /// This method inserts an element into the queue at the given priority level.
    /// Each priority level manages its own set of entities, ensuring fair
    /// scheduling across producers.
    ///
    /// # Arguments
    ///
    /// * `prio` — Priority level of the item (0 = highest priority).
    /// * `entity_id` — Identifier for the logical producer or entity.
    /// * `item` — The data to enqueue.
    ///
    /// # Behavior
    ///
    /// - If the queue is closed (via [`shutdown_immediate()`], [`shutdown_graceful()`], or
    ///   [`shutdown_timeout()`]), this method immediately returns [`PriorityQueueError::Closed`].
    /// - If the provided priority is out of bounds, it returns [`PriorityQueueError::BadPriority`].
    /// - Otherwise, the item is added and all waiting threads are notified with `notify_one()`.
    ///
    /// # Errors
    ///
    /// Returns:
    /// * [`PriorityQueueError::Closed`] — if the queue is closed.
    /// * [`PriorityQueueError::BadPriority`] — if the provided priority index is invalid.
    /// * [`PriorityQueueError::LockError`] — if the internal mutex was poisoned.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pq_sync::SyncPriorityQueue;
    ///
    /// let pq = SyncPriorityQueue::new(3);
    ///
    /// pq.enqueue(0, "client_A".to_string(), "task_1".to_string()).unwrap();
    /// pq.enqueue(1, "client_B".to_string(), "task_2".to_string()).unwrap();
    ///
    /// // Trying to enqueue after shutdown fails:
    /// pq.shutdown_immediate().unwrap();
    /// assert!(pq.enqueue(0, "client_C".to_string(), "task_3".to_string()).is_err());
    /// ```
    ///
    /// # See also
    /// * [`dequeue()`] — Removes an item, blocking if the queue is empty.
    /// * [`try_dequeue()`] — Attempts to remove an item without blocking.
    ///
    pub fn enqueue(&self, prio: usize, entity_id: E, item: T) -> Result<()> {
        let mut st = self
            .inner
            .state
            .lock()
            .map_err(|_e| PriorityQueueError::LockError)?;
        if st.closed {
            return Err(PriorityQueueError::Closed);
        }
        st.pq.enqueue(prio, entity_id, item)?;
        drop(st); // unlock
        self.inner.cv.notify_one();
        Ok(())
    }

    /// Attempts to dequeue an item without blocking.
    ///
    /// This non-blocking variant tries to remove and return the next available
    /// element from the queue. If the queue is empty, it returns `Ok(None)`
    /// immediately.
    ///
    /// # Behavior
    ///
    /// - If the queue contains an item, it is dequeued and returned as `Ok(Some(item))`.
    /// - If the queue becomes empty after removal, all waiting threads are
    ///   notified via `notify_all()`.
    /// - If the queue is empty from the start, the function returns `Ok(None)`
    ///   without blocking.
    ///
    /// # Errors
    ///
    /// Returns:
    /// * [`PriorityQueueError::LockError`] — if the internal mutex was poisoned.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pq_sync::SyncPriorityQueue;
    ///
    /// let pq = SyncPriorityQueue::new(3);
    ///
    /// pq.enqueue(0, "client_A".to_string(), "task_1".to_string()).unwrap();
    ///
    /// assert_eq!(pq.try_dequeue().unwrap(), Some("task_1".to_string()));
    ///
    /// // Nothing left — returns None immediately
    /// assert_eq!(pq.try_dequeue().unwrap(), None);
    /// ```
    ///
    /// # See also
    /// * [`dequeue()`] — Blocking variant that waits for an item.
    /// * [`enqueue()`] — Adds a new item to the queue.
    ///
    pub fn try_dequeue(&self) -> Result<Option<T>> {
        let mut st = self
            .inner
            .state
            .lock()
            .map_err(|_e| PriorityQueueError::LockError)?;
        Ok(st.pq.try_dequeue())
    }

    /// Dequeues an item from the queue, blocking until one becomes available.
    ///
    /// This method waits for an item to be enqueued if the queue is empty,
    /// and returns it once available.
    /// It uses a condition variable (`Condvar`) to sleep efficiently until
    /// new items arrive or the queue is closed.
    ///
    /// # Behavior
    ///
    /// - If the queue has items, the next available one is returned immediately.
    /// - If the queue is empty but **not closed**, the thread blocks until:
    ///   * a producer enqueues a new item, or
    ///   * the queue is closed.
    /// - If the queue is closed **and** empty, it returns [`PriorityQueueError::Closed`].
    /// - If the queue becomes empty after dequeueing, it notifies all waiting threads.
    ///
    /// Internally, this method uses:
    ///
    /// ```ignore
    /// st = self.inner.cv.wait_while(st, |s| s.pq.is_empty() && !s.closed)?;
    /// ```
    ///
    /// This ensures safe handling of **spurious wakeups**, as the condition is
    /// always rechecked upon every wake.
    ///
    /// # Errors
    ///
    /// Returns:
    /// * [`PriorityQueueError::Closed`] — if the queue is closed and empty.
    /// * [`PriorityQueueError::LockError`] — if the internal mutex was poisoned.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pq_sync::SyncPriorityQueue;
    /// use std::thread;
    ///
    /// let pq = SyncPriorityQueue::new(3);
    /// let pq_clone = pq.clone();
    ///
    /// // Consumer
    /// thread::spawn(move || {
    ///     let item = pq_clone.dequeue().unwrap();
    ///     assert_eq!(item, "task_1".to_string());
    /// });
    ///
    /// // Producer
    /// pq.enqueue(0, "client_A".to_string(), "task_1".to_string()).unwrap();
    /// ```
    ///
    /// # See also
    /// * [`try_dequeue()`] — Non-blocking version of this method.
    /// * [`shutdown_graceful()`] — Waits for all items to be consumed before closing.
    /// * [`shutdown_timeout()`] — Same, but with a maximum timeout.
    ///
    pub fn dequeue(&self) -> Result<T> {
        let mut st = self
            .inner
            .state
            .lock()
            .map_err(|_| PriorityQueueError::LockError)?;
        st = self
            .inner
            .cv
            .wait_while(st, |s| s.pq.is_empty() && !s.closed)
            .map_err(|_| PriorityQueueError::LockError)?;
        let Some(v) = st.pq.try_dequeue() else {
            return Err(PriorityQueueError::Closed);
        };
        let became_empty = st.pq.is_empty();
        drop(st);
        if became_empty {
            self.inner.cv.notify_all();
        }
        Ok(v)
    }
}

/// ---
/// ## Shutdown Modes
///
/// Control how and when the queue stops processing items.
///
impl<E, T> SyncPriorityQueue<E, T>
where
    E: Eq + Hash + Clone,
{
    /// Immediately closes the queue and wakes all waiting threads.
    ///
    /// This method sets the internal `closed` flag to `true`, clears all pending
    /// elements in the queue, and notifies all waiting producers and consumers.
    ///
    /// Unlike [`shutdown_graceful()`] or [`shutdown_timeout()`], this variant
    /// does **not wait** for items to be consumed — it simply closes everything
    /// at once.
    ///
    /// # Behavior
    ///
    /// - Any pending calls to [`dequeue()`] will return [`PriorityQueueError::Closed`].
    /// - Any subsequent calls to [`enqueue()`] will also return [`PriorityQueueError::Closed`].
    /// - The internal condition variable is signaled with `notify_all()`,
    ///   ensuring that no thread remains blocked on the queue.
    ///
    /// This operation is **idempotent** — calling it multiple times has no effect
    /// after the first invocation.
    ///
    /// # Errors
    ///
    /// Returns:
    /// * [`PriorityQueueError::LockError`] — if the mutex guarding the internal state
    ///   has been poisoned (for example, due to a panic in another thread).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pq_sync::SyncPriorityQueue;
    ///
    /// let pq = SyncPriorityQueue::new(3);
    ///
    /// pq.enqueue(0, "A".to_string(), "item1".to_string()).unwrap();
    /// pq.enqueue(1, "B".to_string(), "item2".to_string()).unwrap();
    ///
    /// pq.shutdown_immediate().unwrap();
    ///
    /// assert!(pq.enqueue(2, "C".to_string(), "item3".to_string()).is_err());
    /// assert!(pq.dequeue().is_err());
    /// ```
    ///
    /// # See also
    /// * [`shutdown_graceful()`] — Waits for the queue to empty before closing.
    /// * [`shutdown_timeout()`] — Like graceful shutdown, but with a maximum wait duration.
    ///
    pub fn shutdown_immediate(&self) -> Result<()> {
        let mut st = self
            .inner
            .state
            .lock()
            .map_err(|_| PriorityQueueError::LockError)?;
        st.closed = true;
        while st.pq.try_dequeue().is_some() {}
        drop(st);
        self.inner.cv.notify_all();
        Ok(())
    }

    /// Closes the queue and waits until all elements have been consumed.
    ///
    /// This method sets the internal `closed` flag to `true` and blocks
    /// until the queue becomes empty. Once all items are processed,
    /// it wakes all waiting threads with `notify_all()`.
    ///
    /// Unlike [`shutdown_immediate()`], this method gives active consumers
    /// time to finish processing pending elements.
    ///
    /// # Behavior
    ///
    /// - The queue is marked as closed immediately.
    /// - Producers can no longer enqueue new items.
    /// - Consumers may continue dequeuing until the queue is empty.
    /// - Once empty, all waiting threads are notified and the function returns.
    ///
    /// # Errors
    ///
    /// Returns:
    /// * [`PriorityQueueError::LockError`] — if the mutex guarding the internal state
    ///   has been poisoned (for example, due to a panic in another thread).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pq_sync::SyncPriorityQueue;
    /// use std::thread;
    /// use std::time::Duration;
    ///
    /// let pq = SyncPriorityQueue::new(3);
    /// pq.enqueue(0, "A".to_string(), "item1".to_string()).unwrap();
    ///
    /// // A separate thread will dequeue after a short delay.
    /// let pq_clone = pq.clone();
    /// thread::spawn(move || {
    ///     thread::sleep(Duration::from_millis(50));
    ///     pq_clone.dequeue().unwrap();
    /// });
    ///
    /// pq.shutdown_graceful().unwrap();
    /// ```
    ///
    /// # See also
    /// * [`shutdown_immediate()`] — Closes the queue immediately without waiting.
    /// * [`shutdown_timeout()`] — Like graceful shutdown, but with a maximum wait duration.
    ///
    pub fn shutdown_graceful(&self) -> Result<()> {
        let mut st = self
            .inner
            .state
            .lock()
            .map_err(|_| PriorityQueueError::LockError)?;
        st.closed = true;
        if st.pq.is_empty() {
            drop(st);
            self.inner.cv.notify_all();
            return Ok(());
        }
        st = self
            .inner
            .cv
            .wait_while(st, |s| !s.pq.is_empty())
            .map_err(|_| PriorityQueueError::LockError)?;
        drop(st);
        self.inner.cv.notify_all();
        Ok(())
    }

    /// Closes the queue and waits for it to empty, up to a maximum duration.
    ///
    /// This method behaves like [`shutdown_graceful()`],
    /// but limits the total waiting time.
    /// If the queue is not empty after the specified duration,
    /// it returns [`PriorityQueueError::Timeout`].
    ///
    /// # Arguments
    ///
    /// * `timeout` — Maximum duration to wait before aborting the shutdown.
    ///
    /// # Behavior
    ///
    /// - The queue is marked as closed immediately.
    /// - Consumers may continue processing items until the timeout expires.
    /// - If the timeout expires before the queue becomes empty,
    ///   the function returns a `Timeout` error.
    /// - Otherwise, the shutdown completes successfully.
    ///
    /// # Errors
    ///
    /// Returns:
    /// * [`PriorityQueueError::LockError`] — if the mutex guarding the state is poisoned.
    /// * [`PriorityQueueError::Timeout`] — if the queue did not empty within the timeout period.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pq_sync::SyncPriorityQueue;
    /// use std::thread;
    /// use std::time::Duration;
    ///
    /// let pq = SyncPriorityQueue::new(3);
    /// pq.enqueue(0, "A".to_string(), "item1".to_string()).unwrap();
    ///
    /// // Another thread consumes after a short delay
    /// let pq_clone = pq.clone();
    /// thread::spawn(move || {
    ///     thread::sleep(Duration::from_millis(50));
    ///     pq_clone.dequeue().unwrap();
    /// });
    ///
    /// // This will succeed: queue empties before timeout
    /// pq.shutdown_timeout(Duration::from_millis(100)).unwrap();
    ///
    /// // But if we reduce the timeout drastically:
    /// let pq = SyncPriorityQueue::new(3);
    /// pq.enqueue(0, "B".to_string(), "item2".to_string()).unwrap();
    /// let result = pq.shutdown_timeout(Duration::from_millis(5));
    /// assert!(result.is_err()); // -> Timeout
    /// ```
    ///
    /// # See also
    /// * [`shutdown_graceful()`] — Waits indefinitely for the queue to empty.
    /// * [`shutdown_immediate()`] — Closes immediately without waiting.
    ///
    pub fn shutdown_timeout(&self, timeout: Duration) -> Result<()> {
        let mut st = self
            .inner
            .state
            .lock()
            .map_err(|_| PriorityQueueError::LockError)?;
        st.closed = true;
        if st.pq.is_empty() {
            drop(st);
            self.inner.cv.notify_all();
            return Ok(());
        }
        let (next_st, wait_res) = self
            .inner
            .cv
            .wait_timeout_while(st, timeout, |s| !s.pq.is_empty())
            .map_err(|_| PriorityQueueError::LockError)?;

        /*
         * Edge case: race condition between the queue and the timer.
         *
         * If the timeout expires exactly when the queue becomes empty,
         * `wait_timeout_while` may return `timed_out = true` even though
         * the condition is no longer true (the queue is already empty).
         *
         * → Without this extra check, we could incorrectly return `Timeout`.
         *   Always re-test the queue state before deciding.
         */
        let became_empty = next_st.pq.is_empty();
        drop(next_st);
        if wait_res.timed_out() && !became_empty {
            return Err(PriorityQueueError::Timeout);
        }
        self.inner.cv.notify_all();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        thread::{sleep, spawn},
        time::Duration,
    };

    use crate::SyncPriorityQueue;

    #[test]
    fn test_shutdown_timeout_empty() {
        let pq: SyncPriorityQueue<String, String> = SyncPriorityQueue::new(3);
        let res = pq.shutdown_timeout(Duration::from_millis(100));
        assert!(res.is_ok());
    }

    #[test]
    fn test_shutdown_timeout_with_items() {
        let pq: SyncPriorityQueue<String, String> = SyncPriorityQueue::new(3);
        pq.enqueue(0, "A".to_string(), "item1".to_string()).unwrap();

        let pq_clone = pq.clone();
        let handle = spawn(move || {
            sleep(Duration::from_millis(50));
            pq_clone.dequeue().unwrap();
        });

        let res = pq.shutdown_timeout(Duration::from_millis(100));
        assert!(res.is_ok());

        handle.join().unwrap();
    }

    #[test]
    fn test_shutdown_timeout_timeout_reached() {
        let pq: SyncPriorityQueue<String, String> = SyncPriorityQueue::new(3);
        pq.enqueue(0, "A".to_string(), "item1".to_string()).unwrap();

        let res = pq.shutdown_timeout(Duration::from_millis(100));
        assert!(res.is_err());
    }

    #[test]
    fn test_shutdown_graceful() {
        let pq: SyncPriorityQueue<String, String> = SyncPriorityQueue::new(3);
        pq.enqueue(0, "A".to_string(), "item1".to_string()).unwrap();

        let pq_clone = pq.clone();
        let handle = spawn(move || {
            sleep(Duration::from_millis(50));
            pq_clone.dequeue().unwrap();
        });

        let res = pq.shutdown_graceful();
        assert!(res.is_ok());

        handle.join().unwrap();
    }
}
