//!Implementation of [`TaskManager`]
use super::TaskControlBlock;
use crate::sync::UPSafeCell;
use alloc::collections::BinaryHeap;
use alloc::sync::Arc;
use core::cmp::Ordering;
use lazy_static::*;
///A array of `TaskControlBlock` that is thread-safe
pub struct TaskManager {
    ready_queue: BinaryHeap<SchedItem>,
}

struct SchedItem {
    stride: usize,
    pid: usize,
    task: Arc<TaskControlBlock>,
}

impl PartialEq for SchedItem {
    fn eq(&self, other: &Self) -> bool {
        self.stride == other.stride && self.pid == other.pid
    }
}

impl Eq for SchedItem {}

impl PartialOrd for SchedItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SchedItem {
    fn cmp(&self, other: &Self) -> Ordering {
        // BinaryHeap is max-heap, so reverse comparison to make min-stride first.
        other
            .stride
            .cmp(&self.stride)
            .then_with(|| other.pid.cmp(&self.pid))
    }
}

/// A simple FIFO scheduler.
impl TaskManager {
    ///Creat an empty TaskManager
    pub fn new() -> Self {
        Self {
            ready_queue: BinaryHeap::new(),
        }
    }
    /// Add process back to ready queue
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        let inner = task.inner_exclusive_access();
        let item = SchedItem {
            stride: inner.stride,
            pid: task.pid.0,
            task: task.clone(),
        };
        drop(inner);
        self.ready_queue.push(item);
    }
    /// Take a process out of the ready queue
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        let item = self.ready_queue.pop()?;
        let task = item.task;
        {
            let mut inner = task.inner_exclusive_access();
            inner.stride = inner.stride.saturating_add(inner.pass);
        }
        Some(task)
    }
}

lazy_static! {
    /// TASK_MANAGER instance through lazy_static!
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
}

/// Add process to ready queue
pub fn add_task(task: Arc<TaskControlBlock>) {
    //trace!("kernel: TaskManager::add_task");
    TASK_MANAGER.exclusive_access().add(task);
}

/// Take a process out of the ready queue
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    //trace!("kernel: TaskManager::fetch_task");
    TASK_MANAGER.exclusive_access().fetch()
}
