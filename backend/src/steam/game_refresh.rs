use std::collections::HashSet;
use std::sync::Mutex;

/// Appids whose cached rows are stale and wait for the background refresh.
/// An appid is queued once until its refresh finishes, and one worker per
/// process drains the queue.
#[derive(Default)]
pub struct GameRefreshQueue {
    inner: Mutex<QueueState>,
}

#[derive(Default)]
struct QueueState {
    /// Waiting for the worker, in the order they were queued.
    pending: Vec<u32>,
    /// Pending or being refreshed.
    queued: HashSet<u32>,
    worker_running: bool,
}

impl GameRefreshQueue {
    pub fn new() -> Self {
        Self::default()
    }

    /// Queues the `appids` that are not queued already. Returns true when the
    /// caller must start the worker.
    pub fn enqueue(&self, appids: &[u32]) -> bool {
        let mut state = self.inner.lock().expect("refresh queue lock");
        for &appid in appids {
            if state.queued.insert(appid) {
                state.pending.push(appid);
            }
        }
        if state.pending.is_empty() || state.worker_running {
            return false;
        }
        state.worker_running = true;
        true
    }

    /// Takes every pending appid for the worker to refresh. When none are
    /// pending, marks the worker stopped and returns `None`.
    fn take_batch(&self) -> Option<Vec<u32>> {
        let mut state = self.inner.lock().expect("refresh queue lock");
        if state.pending.is_empty() {
            state.worker_running = false;
            return None;
        }
        Some(std::mem::take(&mut state.pending))
    }

    /// The worker is done with `batch`; its appids can be queued again.
    pub fn finish(&self, batch: &[u32]) {
        let mut state = self.inner.lock().expect("refresh queue lock");
        for appid in batch {
            state.queued.remove(appid);
        }
    }

    /// The worker stopped without draining the queue (it panicked or was
    /// cancelled): forget what it was refreshing and let the next caller
    /// start a new worker.
    fn worker_aborted(&self) {
        let mut state = self.inner.lock().expect("refresh queue lock");
        let pending: HashSet<u32> = state.pending.iter().copied().collect();
        state.queued.retain(|appid| pending.contains(appid));
        state.worker_running = false;
    }

    /// Marks the worker stopped if it is dropped before `take_batch` returns
    /// `None`.
    pub fn worker_guard(&self) -> WorkerGuard<'_> {
        WorkerGuard {
            queue: self,
            drained: false,
        }
    }

    /// The appids queued or being refreshed, pending ones in queue order.
    #[cfg(test)]
    pub fn queued(&self) -> Vec<u32> {
        let state = self.inner.lock().expect("refresh queue lock");
        let mut queued = state.pending.clone();
        let mut in_flight: Vec<u32> = state
            .queued
            .iter()
            .copied()
            .filter(|appid| !state.pending.contains(appid))
            .collect();
        in_flight.sort_unstable();
        queued.extend(in_flight);
        queued
    }
}

pub struct WorkerGuard<'a> {
    queue: &'a GameRefreshQueue,
    drained: bool,
}

impl WorkerGuard<'_> {
    /// The next batch to refresh; `None` once the queue is drained.
    pub fn next_batch(&mut self) -> Option<Vec<u32>> {
        let batch = self.queue.take_batch();
        self.drained = batch.is_none();
        batch
    }
}

impl Drop for WorkerGuard<'_> {
    fn drop(&mut self) {
        if !self.drained {
            self.queue.worker_aborted();
        }
    }
}
