use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

/// How long an app Steam failed to refresh stays out of the queue, so a Steam
/// outage or rate limit is not met with a new pass of requests on every list
/// load.
pub(super) const FAILED_REFRESH_COOLDOWN: Duration = Duration::from_secs(15 * 60);

/// Appids whose cached rows are stale and wait for the background refresh.
/// An appid is queued once until its refresh finishes, and one worker per
/// process drains the queue. An appid the refresh failed for is not queued
/// again until a cooldown has passed.
pub struct GameRefreshQueue {
    inner: Mutex<QueueState>,
    /// The time the cooldown is measured with.
    now: Box<dyn Fn() -> Instant + Send + Sync>,
}

impl Default for GameRefreshQueue {
    fn default() -> Self {
        Self {
            inner: Mutex::default(),
            now: Box::new(Instant::now),
        }
    }
}

#[derive(Default)]
struct QueueState {
    /// Waiting for the worker, in the order they were queued.
    pending: Vec<u32>,
    /// Pending or being refreshed; an appid here is not queued again.
    claimed: HashSet<u32>,
    /// When the refresh last failed for an appid still in its cooldown.
    failed_at: HashMap<u32, Instant>,
    worker_running: bool,
}

impl GameRefreshQueue {
    pub fn new() -> Self {
        Self::default()
    }

    /// A queue whose cooldown is measured with `clock` instead of the system
    /// clock.
    #[cfg(test)]
    pub fn with_clock(clock: ManualClock) -> Self {
        Self {
            inner: Mutex::default(),
            now: Box::new(move || clock.now()),
        }
    }

    /// The queue state. Every update leaves it consistent, so a lock poisoned
    /// by a panic elsewhere is still safe to use.
    fn state(&self) -> MutexGuard<'_, QueueState> {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Queues the `appids` that are not claimed already or in their cooldown.
    /// Returns true when the caller must start the worker with `start_worker`.
    #[must_use = "when true, the caller must start the refresh worker"]
    pub fn enqueue(&self, appids: &[u32]) -> bool {
        let now = (self.now)();
        let mut state = self.state();
        state
            .failed_at
            .retain(|_, failed_at| now.duration_since(*failed_at) < FAILED_REFRESH_COOLDOWN);
        for &appid in appids {
            if state.failed_at.contains_key(&appid) {
                continue;
            }
            if state.claimed.insert(appid) {
                state.pending.push(appid);
            }
        }
        if state.pending.is_empty() || state.worker_running {
            return false;
        }
        state.worker_running = true;
        true
    }

    /// The worker's handle on the queue, for the caller `enqueue` told to
    /// start it.
    pub fn start_worker(&self) -> RefreshWorker<'_> {
        RefreshWorker {
            queue: self,
            batch: Vec::new(),
            drained: false,
        }
    }

    /// The appids queued or being refreshed, pending ones in queue order.
    #[cfg(test)]
    pub fn queued(&self) -> Vec<u32> {
        let state = self.state();
        let mut queued = state.pending.clone();
        let mut in_flight: Vec<u32> = state
            .claimed
            .iter()
            .copied()
            .filter(|appid| !state.pending.contains(appid))
            .collect();
        in_flight.sort_unstable();
        queued.extend(in_flight);
        queued
    }
}

/// The one worker draining a `GameRefreshQueue`. Dropping it, whether it
/// drained the queue, panicked or was cancelled, releases the batch it held
/// and lets the next `enqueue` start a new worker.
pub struct RefreshWorker<'a> {
    queue: &'a GameRefreshQueue,
    batch: Vec<u32>,
    drained: bool,
}

impl RefreshWorker<'_> {
    /// Starts the cooldown for `appids`, which the refresh failed for. They are
    /// not queued again until it has passed.
    pub fn record_failed(&mut self, appids: &[u32]) {
        let now = (self.queue.now)();
        let mut state = self.queue.state();
        for &appid in appids {
            state.failed_at.insert(appid, now);
        }
    }

    /// Finishes the previous batch and takes every pending appid as the next
    /// one; `None` once the queue is drained, which stops the worker.
    pub fn next_batch(&mut self) -> Option<Vec<u32>> {
        let mut state = self.queue.state();
        for appid in self.batch.drain(..) {
            state.claimed.remove(&appid);
        }
        if state.pending.is_empty() {
            state.worker_running = false;
            self.drained = true;
            return None;
        }
        self.batch = std::mem::take(&mut state.pending);
        Some(self.batch.clone())
    }
}

impl Drop for RefreshWorker<'_> {
    fn drop(&mut self) {
        if self.drained {
            return;
        }
        let mut state = self.queue.state();
        for appid in &self.batch {
            state.claimed.remove(appid);
        }
        state.worker_running = false;
    }
}

#[cfg(test)]
pub use manual_clock::ManualClock;

#[cfg(test)]
mod manual_clock {
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};

    /// A clock that only moves when a test advances it.
    #[derive(Clone)]
    pub struct ManualClock(Arc<Mutex<Instant>>);

    impl Default for ManualClock {
        fn default() -> Self {
            Self(Arc::new(Mutex::new(Instant::now())))
        }
    }

    impl ManualClock {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn now(&self) -> Instant {
            *self.0.lock().unwrap()
        }

        pub fn advance(&self, by: Duration) {
            *self.0.lock().unwrap() += by;
        }
    }
}
