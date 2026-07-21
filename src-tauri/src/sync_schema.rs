//! Sync schema stubs for a later cut — types only, no runtime.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncLogEntry {
    pub id: Uuid,
    pub entity: String,
    pub entity_id: String,
    pub op: String,
    pub payload_hash: String,
    pub created_at: DateTime<Utc>,
}

#[allow(dead_code)]
pub const SYNC_SCHEMA_VERSION: u32 = 1;
