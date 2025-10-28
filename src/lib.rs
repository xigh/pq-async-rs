use std::{collections::VecDeque, fmt, result};

struct PriorityQueue<T> {
    queues: Vec<VecDeque<T>>,
}

impl<T> PriorityQueue<T> {
    // fn new
    pub fn new(n_prio: usize) -> Self {
        let mut queues = Vec::with_capacity(n_prio);
        queues.resize_with(n_prio, VecDeque::<T>::new);
        Self { queues }
    }

    // fn enqueue
    pub fn enqueue(&mut self, prio: usize, item: T) -> Result<()> {
        if prio >= self.queues.len() {
            return Err(PriorityQueueError::BadPriority(prio));
        }
        self.queues[prio].push_back(item);
        Ok(())
    }

    // fn try_dequeue
    pub fn try_dequeue(&mut self) -> Option<T> {
        for p in 0..self.queues.len() {
            if let Some(item) = self.queues[p].pop_front() {
                return Some(item);
            }
        }
        None
    }
}

type Result<T> = result::Result<T, PriorityQueueError>;

enum PriorityQueueError {
    BadPriority(usize),
    NotImplemented,
}

impl fmt::Display for PriorityQueueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PriorityQueueError::BadPriority(prio) => write!(f, "bad priority {}", prio),
            PriorityQueueError::NotImplemented => write!(f, "not implemented"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut pq = PriorityQueue::new(3);

        assert!(pq.try_dequeue().is_none());

        assert!(pq.enqueue(1, "hello world".to_string()).is_ok());

        assert!(pq.enqueue(0, "foobar".to_string()).is_ok());

        // dequeue foobar

        let res = pq.try_dequeue();
        assert!(res.is_some());

        let task = res.unwrap();
        assert_eq!(task.as_str(), "foobar");

        // dequeue hello

        let res = pq.try_dequeue();
        assert!(res.is_some());

        let task = res.unwrap();
        assert_eq!(task.as_str(), "hello world");

        // pq must be empty

        assert!(pq.try_dequeue().is_none());
    }
}
