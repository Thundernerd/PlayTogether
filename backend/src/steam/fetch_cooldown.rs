use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

/// How long an app Steam failed for is not fetched again, so a Steam outage
/// or rate limit is not met with a new pass of requests on every list load.
/// It covers the background refresh of stale rows and the fetch of apps with
/// no row that list loads wait for.
pub(super) const FAILED_FETCH_COOLDOWN: Duration = Duration::from_secs(15 * 60);

/// The cooldown for apps a fetch from Steam failed for, whether the
/// background refresh or a list load's fetch of an app with no row: such an
/// appid is not queued for refresh or fetched again until it has passed.
pub struct FetchCooldown {
    /// When a fetch from Steam last failed for an appid still in its cooldown.
    failed_at: Mutex<HashMap<u32, Instant>>,
    /// The time the cooldown is measured with.
    now: Box<dyn Fn() -> Instant + Send + Sync>,
}

impl Default for FetchCooldown {
    fn default() -> Self {
        Self {
            failed_at: Mutex::default(),
            now: Box::new(Instant::now),
        }
    }
}

impl FetchCooldown {
    pub fn new() -> Self {
        Self::default()
    }

    /// A cooldown measured with `clock` instead of the system clock.
    #[cfg(test)]
    pub fn with_clock(clock: ManualClock) -> Self {
        Self {
            failed_at: Mutex::default(),
            now: Box::new(move || clock.now()),
        }
    }

    /// The failure times. Every update leaves them consistent, so a lock
    /// poisoned by a panic elsewhere is still safe to use.
    fn failed_at(&self) -> MutexGuard<'_, HashMap<u32, Instant>> {
        self.failed_at
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    /// Starts the cooldown for `appids`, which a fetch from Steam failed for.
    /// They are not queued or fetched again until it has passed.
    pub fn record_failed(&self, appids: &[u32]) {
        let now = (self.now)();
        let mut failed_at = self.failed_at();
        for &appid in appids {
            failed_at.insert(appid, now);
        }
    }

    /// The `appids` not in their cooldown, in the order given.
    pub fn not_cooling_down(&self, appids: &[u32]) -> Vec<u32> {
        let now = (self.now)();
        let mut failed_at = self.failed_at();
        failed_at.retain(|_, failed_at| now.duration_since(*failed_at) < FAILED_FETCH_COOLDOWN);
        appids
            .iter()
            .copied()
            .filter(|appid| !failed_at.contains_key(appid))
            .collect()
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
