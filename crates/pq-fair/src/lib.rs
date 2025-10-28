use std::{
    collections::{HashMap, HashSet, VecDeque},
    hash::Hash,
};

use pq_core::{PriorityQueueError, Result};

struct PriorityLevel<E, T>
where
    E: Eq + Hash + Clone,
{
    by_entities: HashMap<E, VecDeque<T>>,
    rr: VecDeque<E>,
    actives: HashSet<E>,
}

pub struct PriorityQueue<E, T>
where
    E: Eq + Hash + Clone,
{
    queues: Vec<PriorityLevel<E, T>>,
}

impl<E, T> PriorityLevel<E, T>
where
    E: Eq + Hash + Clone,
{
    pub fn new() -> Self {
        Self {
            by_entities: HashMap::new(),
            rr: VecDeque::new(),
            actives: HashSet::new(),
        }
    }
}

impl<E, T> PriorityQueue<E, T>
where
    E: Eq + Hash + Clone,
{
    // fn new
    pub fn new(n_prio: usize) -> Self {
        let mut queues = Vec::with_capacity(n_prio);
        queues.resize_with(n_prio, PriorityLevel::new);
        Self { queues }
    }

    // fn enqueue
    pub fn enqueue(&mut self, prio: usize, entity_id: E, item: T) -> Result<()> {
        if prio >= self.queues.len() {
            return Err(PriorityQueueError::BadPriority(prio));
        }
        let level = &mut self.queues[prio];
        if level.actives.insert(entity_id.clone()) {
            level.rr.push_back(entity_id.clone());
        }
        level
            .by_entities
            .entry(entity_id)
            .or_default()
            .push_back(item);

        Ok(())
    }

    // fn try_dequeue
    pub fn try_dequeue(&mut self) -> Option<T> {
        // for each level
        for level in self.queues.iter_mut() {
            // if there is an entity in round-robin deque
            if let Some(entity_id) = level.rr.pop_front() {
                // look if there is a task/item available
                if let Some(items) = level.by_entities.get_mut(&entity_id) {
                    if let Some(item) = items.pop_front() {
                        if !items.is_empty() {
                            level.rr.push_back(entity_id);
                        } else {
                            level.by_entities.remove(&entity_id);
                            level.actives.remove(&entity_id);
                        }
                        // println!("{}", items.len()); // <- not allowed by the compiler
                        return Some(item);
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut pq = PriorityQueue::new(3);

        assert!(pq.try_dequeue().is_none());

        let a = "A".to_string();
        let b = "B".to_string();
        let c = "C".to_string();

        for i in 1..=4 {
            let res = pq.enqueue(1, a.clone(), format!("{}{}", a, i));
            assert!(res.is_ok());
        }

        for i in 1..=2 {
            let res = pq.enqueue(1, b.clone(), format!("{}{}", b, i));
            assert!(res.is_ok());
        }

        let res = pq.enqueue(0, c.clone(), format!("{}1", c));
        assert!(res.is_ok());

        // 1. expected C1
        let ret = pq.try_dequeue();
        assert!(ret.is_some() && ret.unwrap() == "C1");

        // 2. expected A1, B1, A2, B2, A3, A4
        let ret = pq.try_dequeue();
        assert!(ret.is_some() && ret.unwrap() == "A1");
        let ret = pq.try_dequeue();
        assert!(ret.is_some() && ret.unwrap() == "B1");
        let ret = pq.try_dequeue();
        assert!(ret.is_some() && ret.unwrap() == "A2");
        let ret = pq.try_dequeue();
        assert!(ret.is_some() && ret.unwrap() == "B2");
        let ret = pq.try_dequeue();
        assert!(ret.is_some() && ret.unwrap() == "A3");
        let ret = pq.try_dequeue();
        assert!(ret.is_some() && ret.unwrap() == "A4");

        // pq must be empty

        assert!(pq.try_dequeue().is_none());
    }
}
