//! # ftsim-proto::protocols::primary_backup
//!
//! An example implementation of a simple Primary-Backup replication protocol.
//! This demonstrates the basic usage of the `Protocol<M>` SDK.

use crate::{Ctx, FaultEvent, Protocol};
use ftsim_types::{
    envelope::ProtoTag,
    id::{NodeId, TimerId},
};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

const TAG: ProtoTag = ProtoTag(2);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Message {
    WriteRequest { key: String, value: String },
    Ack { key: String },
    StateUpdate { state: IndexMap<String, String> },
}

#[derive(Default)]
pub struct PrimaryBackup {
    id: NodeId,
    primary: NodeId,
    is_primary: bool,
    peers: Vec<NodeId>,
    data: IndexMap<String, String>,
}

impl PrimaryBackup {
    pub fn new() -> Self {
        Self {
            primary: 0, // Node 0 is the initial primary
            ..Default::default()
        }
    }
}

impl Protocol<Message> for PrimaryBackup {
    fn name(&self) -> &'static str {
        "primary_backup"
    }

    fn proto_tag(&self) -> ProtoTag {
        TAG
    }

    fn init(&mut self, ctx: &mut Ctx<Message>) {
        self.id = ctx.node_id();
        self.is_primary = self.id == self.primary;
        // A real implementation would discover peers. Here we assume a fixed set.
        self.peers = (0..3).filter(|&i| i != self.id).collect();
        let role = if self.is_primary { "primary" } else { "backup" };
        ctx.log_kv("role", role);
        ctx.log_kv("data_entries", &self.data.len().to_string());
        tracing::info!(node_id = self.id, role = role, peers = ?self.peers, "🔧 Primary-backup node initialized");
    }

    fn on_message(&mut self, ctx: &mut Ctx<Message>, src: NodeId, msg: Message) {
        match msg {
            Message::WriteRequest { key, value } => {
                if self.is_primary {
                    tracing::info!(node_id = self.id, key = %key, value = %value, "✍️  PRIMARY: Processing write request");
                    self.data.insert(key.clone(), value.clone());
                    ctx.log_kv("data_entries", &self.data.len().to_string());
                    ctx.log_kv("last_write_key", &key);
                    
                    // Replicate to backups
                    let update = Message::StateUpdate {
                        state: self.data.clone(),
                    };
                    tracing::info!(node_id = self.id, peers = ?self.peers, "📡 PRIMARY: Replicating state to backups");
                    ctx.broadcast(&update, None).ok();
                    
                    // Acknowledge the original sender
                    tracing::info!(node_id = self.id, src = src, key = %key, "✅ PRIMARY: Sending acknowledgment");
                    ctx.send(src, &Message::Ack { key }).ok();
                } else {
                    tracing::warn!(node_id = self.id, key = %key, "❌ BACKUP: Received write request, should go to primary");
                }
            }
            Message::StateUpdate { state } => {
                if !self.is_primary {
                    let old_size = self.data.len();
                    let new_size = state.len();
                    tracing::info!(node_id = self.id, old_entries = old_size, new_entries = new_size, "🔄 BACKUP: Received state update from primary");
                    self.data = state;
                    ctx.log_kv("data_entries", &self.data.len().to_string());
                    if let Some((last_key, _)) = self.data.last() {
                        ctx.log_kv("last_key", last_key);
                    }
                } else {
                    tracing::warn!(node_id = self.id, "❌ PRIMARY: Received state update, ignoring");
                }
            }
            Message::Ack { key } => {
                tracing::info!(node_id = self.id, src = src, key = %key, "✅ Received write acknowledgment");
            }
        }
    }

    fn on_timer(&mut self, _ctx: &mut Ctx<Message>, _timer: TimerId) {
        // Not used in this simple protocol
    }

    fn on_fault(&mut self, ctx: &mut Ctx<Message>, fault: FaultEvent) {
        match fault {
            FaultEvent::NodeCrashed => {
                tracing::warn!(node_id = self.id, role = if self.is_primary { "primary" } else { "backup" }, "💥 Node crashed - entering recovery mode");
                ctx.log_kv("fault", "crash_detected");
                ctx.log_kv("status", "crashed");
            }
            FaultEvent::NodeRecovered => {
                tracing::info!(node_id = self.id, role = if self.is_primary { "primary" } else { "backup" }, "🔄 Node recovered from crash");
                ctx.log_kv("status", "recovered");
                // Re-initialize state tracking
                ctx.log_kv("data_entries", &self.data.len().to_string());
            }
            _ => {
                tracing::info!(node_id = self.id, ?fault, "⚠️  Other fault event received");
            }
        }
    }
}
