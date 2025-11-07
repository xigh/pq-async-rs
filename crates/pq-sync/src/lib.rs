use std::{
    hash::Hash,
    sync::{Arc, Condvar, Mutex},
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
    pub fn new(n_prio: usize) -> Self {
        Self {
            inner: Arc::new(Inner::new(n_prio)),
        }
    }

    // fn enqueue
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

    // fn try_dequeue
    pub fn try_dequeue(&mut self) -> Result<Option<T>> {
        let mut st = self
            .inner
            .state
            .lock()
            .map_err(|_e| PriorityQueueError::LockError)?;
        Ok(st.pq.try_dequeue())
    }

    pub fn dequeue(&mut self) -> Result<T> {
        let mut st = self
            .inner
            .state
            .lock()
            .map_err(|_| PriorityQueueError::LockError)?;
        loop {
            if let Some(v) = st.pq.try_dequeue() {
                return Ok(v);
            }
            if st.closed {
                return Err(PriorityQueueError::Closed);
            }
            st = self
                .inner
                .cv
                .wait(st)
                .map_err(|_| PriorityQueueError::LockError)?;
        }
    }

    // fn shutdown_immediate
    pub fn shutdown_immediate(&self) -> Result<()> {
        let mut st = self
            .inner
            .state
            .lock()
            .map_err(|_| PriorityQueueError::LockError)?;
        st.closed = true;
        while let Some(_) = st.pq.try_dequeue() {}
        drop(st);
        self.inner.cv.notify_all();
        Ok(())
    }
}
