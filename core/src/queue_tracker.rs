use crate::queue::Queue;
use crate::queue_entry::QueueEntry;


struct QueueTracker {
    queues: Vec<Box<Queue<dyn QueueEntry>>>,
}