use std::{
    hash::Hash,
    sync::{Arc, Mutex},
};

use pq_core::{PriorityQueueError, Result};
use pq_fair::PriorityQueue;

#[derive(Clone)]
pub struct SyncPriorityQueue<E, T>
where
    E: Eq + Hash + Clone,
{
    inner: Arc<Mutex<PriorityQueue<E, T>>>,
}

impl<E, T> SyncPriorityQueue<E, T>
where
    E: Eq + Hash + Clone,
{
    pub fn new(n_prio: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(PriorityQueue::new(n_prio))),
        }
    }

    // fn enqueue
    pub fn enqueue(&self, prio: usize, entity_id: E, item: T) -> Result<()> {
        let mut pq = self
            .inner
            .lock()
            .map_err(|_e| PriorityQueueError::LockError)?;
        pq.enqueue(prio, entity_id, item)
    }

    // fn try_dequeue
    pub fn try_dequeue(&mut self) -> Result<Option<T>> {
        let mut pq = self
            .inner
            .lock()
            .map_err(|_e| PriorityQueueError::LockError)?;
        Ok(pq.try_dequeue())
    }
}
