use std::{fmt, result};

pub type Result<T> = result::Result<T, PriorityQueueError>;

#[derive(Debug)]
pub enum PriorityQueueError {
    BadPriority(usize),
    LockError,
    Closed,
    Timeout,
    NotImplemented,
}

impl fmt::Display for PriorityQueueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PriorityQueueError::BadPriority(prio) => write!(f, "bad priority {}", prio),
            PriorityQueueError::LockError => write!(f, "lock failed"),
            PriorityQueueError::Closed => write!(f, "closed"),
            PriorityQueueError::Timeout => write!(f, "timeout"),
            PriorityQueueError::NotImplemented => write!(f, "not implemented"),
        }
    }
}
