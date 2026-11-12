use std::future::Future;

use crate::error::AppResult;
use crate::steam::client::{AppDetailsData, SteamClient, StoreItem};

/// The Steam store calls the game cache makes. `SteamClient` talks to Steam;
/// tests use `fake::FakeSteam`.
pub trait SteamStore {
    /// `GetItems` with the list data request. Steam may answer an app with an
    /// item for a different appid (0 for delisted or hidden apps).
    fn fetch_store_items(
        &self,
        appids: &[u32],
    ) -> impl Future<Output = AppResult<Vec<StoreItem>>> + Send;

    /// `GetItems` with the detail data request, for one app.
    fn fetch_store_item_details(
        &self,
        appid: u32,
    ) -> impl Future<Output = AppResult<Vec<StoreItem>>> + Send;

    /// `appdetails` for one app; `None` when Steam does not know it.
    fn fetch_app_details(
        &self,
        appid: u32,
    ) -> impl Future<Output = AppResult<Option<AppDetailsData>>> + Send;
}

impl SteamStore for SteamClient {
    fn fetch_store_items(
        &self,
        appids: &[u32],
    ) -> impl Future<Output = AppResult<Vec<StoreItem>>> + Send {
        SteamClient::fetch_store_items(self, appids)
    }

    fn fetch_store_item_details(
        &self,
        appid: u32,
    ) -> impl Future<Output = AppResult<Vec<StoreItem>>> + Send {
        SteamClient::fetch_store_item_details(self, appid)
    }

    fn fetch_app_details(
        &self,
        appid: u32,
    ) -> impl Future<Output = AppResult<Option<AppDetailsData>>> + Send {
        SteamClient::fetch_app_details(self, appid)
    }
}

#[cfg(test)]
pub mod fake {
    use std::collections::{HashMap, HashSet};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    use tokio::sync::Semaphore;

    use super::*;
    use crate::error::AppError;

    /// An in-memory Steam store. Apps are unknown unless added with one of the
    /// builder methods.
    #[derive(Default)]
    pub struct FakeSteam {
        store_items: HashMap<u32, StoreItem>,
        app_details: HashMap<u32, AppDetailsData>,
        hidden: HashSet<u32>,
        failing_every_call: HashSet<u32>,
        failing_app_details: HashSet<u32>,
        failing_when_alone: HashSet<u32>,
        rate_limited: bool,
        rate_limited_when_alone: bool,
        calls: AtomicUsize,
        gate: Option<Arc<Semaphore>>,
    }

    /// Releases the calls a `FakeSteam` built with `held` holds.
    pub struct SteamGate(Arc<Semaphore>);

    impl SteamGate {
        /// Lets every held call, and every later one, answer.
        pub fn release(&self) {
            self.0.close();
        }
    }

    impl FakeSteam {
        pub fn new() -> Self {
            Self::default()
        }

        /// `GetItems` knows the app, with this name.
        pub fn with_store_item(mut self, appid: u32, name: &str) -> Self {
            let item = serde_json::from_value(serde_json::json!({ "appid": appid, "name": name }))
                .expect("valid StoreItem");
            self.store_items.insert(appid, item);
            self
        }

        /// Only `appdetails` knows the app, with this name.
        pub fn with_app_details(mut self, appid: u32, name: &str) -> Self {
            let detail =
                serde_json::from_value(serde_json::json!({ "name": name, "steam_appid": appid }))
                    .expect("valid AppDetailsData");
            self.app_details.insert(appid, detail);
            self
        }

        /// `GetItems` answers the app the way Steam answers delisted or hidden
        /// apps: with an item whose appid is 0.
        pub fn with_hidden_app(mut self, appid: u32) -> Self {
            self.hidden.insert(appid);
            self
        }

        /// Every call that asks for the app fails, as an HTTP error would.
        pub fn failing_every_call_for(mut self, appid: u32) -> Self {
            self.failing_every_call.insert(appid);
            self
        }

        /// `appdetails` fails for the app, as an HTTP error would; `GetItems`
        /// answers it as usual.
        pub fn failing_app_details_for(mut self, appid: u32) -> Self {
            self.failing_app_details.insert(appid);
            self
        }

        /// `GetItems` fails when asked for the app on its own, and leaves it out
        /// of the answer when asked for it with other apps.
        pub fn failing_when_asked_alone_for(mut self, appid: u32) -> Self {
            self.failing_when_alone.insert(appid);
            self
        }

        /// Every `GetItems` call fails the way Steam answers with HTTP 429.
        pub fn rate_limiting_every_store_items_call(mut self) -> Self {
            self.rate_limited = true;
            self
        }

        /// `GetItems` fails the way Steam answers with HTTP 429 when asked for
        /// one app on its own.
        pub fn rate_limiting_store_items_calls_for_one_app(mut self) -> Self {
            self.rate_limited_when_alone = true;
            self
        }

        /// Every call waits until the returned gate is released before it
        /// answers, as calls to a slow Steam would.
        pub fn held(mut self) -> (Self, SteamGate) {
            let gate = Arc::new(Semaphore::new(0));
            self.gate = Some(gate.clone());
            (self, SteamGate(gate))
        }

        /// How many calls were made to Steam, held ones included.
        pub fn calls(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }

        /// Counts the call and waits for the gate, if the fake is held.
        async fn call(&self) {
            self.calls.fetch_add(1, Ordering::SeqCst);
            if let Some(gate) = &self.gate {
                // The gate is released by closing it, so acquiring fails then.
                let _ = gate.acquire().await;
            }
        }

        fn fail_if_any_failing(&self, appids: &[u32]) -> AppResult<()> {
            match appids
                .iter()
                .find(|appid| self.failing_every_call.contains(appid))
            {
                Some(appid) => Err(AppError::Steam(format!(
                    "fake Steam failure for app {appid}"
                ))),
                None => Ok(()),
            }
        }

        fn store_items_for(&self, appids: &[u32]) -> AppResult<Vec<StoreItem>> {
            if self.rate_limited || (self.rate_limited_when_alone && appids.len() == 1) {
                return Err(AppError::SteamRateLimited(
                    "fake GetItems rate limit".into(),
                ));
            }
            self.fail_if_any_failing(appids)?;
            if let [appid] = appids {
                if self.failing_when_alone.contains(appid) {
                    return Err(AppError::Steam(format!(
                        "fake GetItems failure for app {appid}"
                    )));
                }
            }
            Ok(appids
                .iter()
                .filter_map(|appid| {
                    if self.hidden.contains(appid) {
                        Some(
                            serde_json::from_value(
                                serde_json::json!({ "appid": 0, "success": 15 }),
                            )
                            .expect("valid StoreItem"),
                        )
                    } else {
                        self.store_items.get(appid).cloned()
                    }
                })
                .collect())
        }
    }

    impl SteamStore for FakeSteam {
        async fn fetch_store_items(&self, appids: &[u32]) -> AppResult<Vec<StoreItem>> {
            self.call().await;
            self.store_items_for(appids)
        }

        async fn fetch_store_item_details(&self, appid: u32) -> AppResult<Vec<StoreItem>> {
            self.call().await;
            self.store_items_for(&[appid])
        }

        async fn fetch_app_details(&self, appid: u32) -> AppResult<Option<AppDetailsData>> {
            self.call().await;
            self.fail_if_any_failing(&[appid])?;
            if self.failing_app_details.contains(&appid) {
                return Err(AppError::Steam(format!(
                    "fake appdetails failure for app {appid}"
                )));
            }
            Ok(self.app_details.get(&appid).cloned())
        }
    }
}
