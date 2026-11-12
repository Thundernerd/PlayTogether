use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, MutexGuard, PoisonError};

use tokio::sync::watch;

use crate::steam::fetch_cooldown::FetchCooldown;

/// Appids whose cached rows are stale and wait for the background refresh.
/// An appid is queued once until its refresh finishes, and one worker per
/// process drains the queue. It also holds the apps with no row that list
/// loads are fetching, so overlapping list loads fetch such an app once.
#[derive(Default)]
pub struct GameRefreshQueue {
    inner: Mutex<QueueState>,
}

#[derive(Default)]
struct QueueState {
    /// Waiting for the worker, in the order they were queued.
    pending: Vec<u32>,
    /// Pending or being refreshed; an appid here is not queued again.
    claimed: HashSet<u32>,
    worker_running: bool,
    /// Appids with no row that a list load is fetching from Steam. Removing
    /// an appid closes its channel, which wakes the list loads waiting for it.
    fetching: HashMap<u32, watch::Sender<()>>,
}

impl GameRefreshQueue {
    pub fn new() -> Self {
        Self::default()
    }

    /// The queue state. Every update leaves it consistent, so a lock poisoned
    /// by a panic elsewhere is still safe to use.
    fn state(&self) -> MutexGuard<'_, QueueState> {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Claims the fetch of the `appids`, which have no row, for one list load.
    /// The ones another list load is fetching already are not claimed again;
    /// the claim holds a wait for each of their fetches instead.
    pub fn claim_fetches(&self, appids: &[u32]) -> FetchClaim<'_> {
        let mut state = self.state();
        let mut owned = Vec::new();
        let mut in_flight_elsewhere = Vec::new();
        for &appid in appids {
            match state.fetching.get(&appid) {
                Some(fetch) => in_flight_elsewhere.push(InFlightFetch(fetch.subscribe())),
                None => {
                    state.fetching.insert(appid, watch::channel(()).0);
                    owned.push(appid);
                }
            }
        }
        FetchClaim {
            fetch: OwnedFetch {
                queue: self,
                appids: owned,
            },
            in_flight_elsewhere,
        }
    }

    /// How many list loads wait for another list load's fetch of `appid`.
    #[cfg(test)]
    pub fn waiting_for(&self, appid: u32) -> usize {
        self.state()
            .fetching
            .get(&appid)
            .map_or(0, watch::Sender::receiver_count)
    }

    /// Queues the `appids` that are not claimed already or in their
    /// `cooldown`. Returns true when the caller must start the worker with
    /// `start_worker`.
    #[must_use = "when true, the caller must start the refresh worker"]
    pub fn enqueue(&self, appids: &[u32], cooldown: &FetchCooldown) -> bool {
        let appids = cooldown.not_cooling_down(appids);
        let mut state = self.state();
        for appid in appids {
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

/// What `GameRefreshQueue::claim_fetches` gave a list load: the fetches it
/// owns and the other list loads' fetches it waits for.
pub struct FetchClaim<'a> {
    pub fetch: OwnedFetch<'a>,
    pub in_flight_elsewhere: Vec<InFlightFetch>,
}

/// The appids one list load fetches from Steam. `release` gives up the claim
/// on the ones the fetch has settled. Dropping it, whether the fetch
/// finished, failed or was cancelled, releases the rest. Releasing an appid
/// wakes the list loads waiting for it.
pub struct OwnedFetch<'a> {
    queue: &'a GameRefreshQueue,
    appids: Vec<u32>,
}

impl OwnedFetch<'_> {
    /// The appids this list load fetches; empty when other list loads are
    /// fetching all of the ones it asked for.
    pub fn appids(&self) -> &[u32] {
        &self.appids
    }

    /// Releases the claim on the `appids` this list load is done fetching,
    /// before the rest of its fetch has finished. Appids it does not hold are
    /// ignored, so a claim another list load has taken since stays.
    pub fn release(&mut self, appids: &[u32]) {
        let mut state = self.queue.state();
        self.appids.retain(|appid| {
            let settled = appids.contains(appid);
            if settled {
                state.fetching.remove(appid);
            }
            !settled
        });
    }
}

impl Drop for OwnedFetch<'_> {
    fn drop(&mut self) {
        let mut state = self.queue.state();
        for appid in &self.appids {
            state.fetching.remove(appid);
        }
    }
}

/// Another list load's fetch of an app with no row.
pub struct InFlightFetch(watch::Receiver<()>);

impl InFlightFetch {
    /// Waits until the fetch has ended, however it ended.
    pub async fn ended(mut self) {
        // Nothing is sent on the channel, so this returns once it closes.
        let _ = self.0.changed().await;
    }
}
