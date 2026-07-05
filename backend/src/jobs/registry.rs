use std::collections::HashMap;

use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use uuid::Uuid;

#[derive(Default)]
pub struct SyncJobRegistry {
    handles: RwLock<HashMap<Uuid, JoinHandle<()>>>,
}

impl SyncJobRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn insert(&self, member_id: Uuid, handle: JoinHandle<()>) {
        let mut handles = self.handles.write().await;
        if let Some(existing) = handles.remove(&member_id) {
            existing.abort();
        }
        handles.insert(member_id, handle);
    }

    pub async fn remove(&self, member_id: Uuid) {
        self.handles.write().await.remove(&member_id);
    }

    pub async fn cancel(&self, member_id: Uuid) {
        if let Some(handle) = self.handles.write().await.remove(&member_id) {
            handle.abort();
        }
    }

    pub async fn cancel_many(&self, member_ids: &[Uuid]) {
        for member_id in member_ids {
            self.cancel(*member_id).await;
        }
    }
}
