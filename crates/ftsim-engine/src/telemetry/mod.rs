//! # ftsim-engine::telemetry
//!
//! The observability subsystem. It is responsible for collecting and
//! dispatching logs, metrics, and state snapshots.

use crate::{prelude::*, world::World};
use crossbeam_channel::Sender;
use indexmap::IndexMap;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

pub mod snapshot;
pub mod tracing_layer;

/// A central bus for telemetry data.
/// It uses channels to communicate with external consumers (like the TUI)
/// and a shared state for contextual logging.
#[derive(Clone)]
pub struct TelemetryBus {
    snapshot_tx: Sender<Snapshot>,
    // Shared state for the tracing layer to access simulation context.
    context: Arc<Mutex<TracingContext>>,
}

#[derive(Default)]
struct TracingContext {
    time: SimTime,
    event_id: EventId,
    // Per-node custom KVs from protocols
    node_kvs: Vec<IndexMap<String, Value>>,
    // Recent events for visualization (keep last 100)
    recent_events: VecDeque<snapshot::LogSnap>,
    // Running metrics
    metrics: snapshot::MetricsSnapshot,
}

impl TelemetryBus {
    pub fn new(snapshot_tx: Sender<Snapshot>, num_nodes: usize) -> Self {
        Self {
            snapshot_tx,
            context: Arc::new(Mutex::new(TracingContext {
                time: 0,
                event_id: 0,
                node_kvs: vec![IndexMap::new(); num_nodes],
                recent_events: VecDeque::with_capacity(100),
                metrics: snapshot::MetricsSnapshot::default(),
            })),
        }
    }

    pub fn send_snapshot(&self, snap: Snapshot) {
        // Try sending, but don't block if the TUI is not consuming.
        let _ = self.snapshot_tx.try_send(snap);
    }

    pub fn set_current_time(&self, time: SimTime, event_id: EventId) {
        let mut ctx = self.context.lock().unwrap();
        ctx.time = time;
        ctx.event_id = event_id;
    }

    pub fn log_node_kv(&self, node_id: NodeId, key: String, val: Value) {
        let mut ctx = self.context.lock().unwrap();
        if let Some(map) = ctx.node_kvs.get_mut(node_id as usize) {
            map.insert(key, val);
        }
    }

    pub(crate) fn context(&self) -> Arc<Mutex<TracingContext>> {
        self.context.clone()
    }

    /// Logs a simulation event for visualization.
    pub fn log_event(&self, event_type: String, details: String, node_id: Option<NodeId>) {
        let mut ctx = self.context.lock().unwrap();
        let log_snap = snapshot::LogSnap {
            event_id: ctx.event_id,
            time: ctx.time,
            event_type,
            details,
            node_id,
        };
        
        // Keep only the last 100 events
        if ctx.recent_events.len() >= 100 {
            ctx.recent_events.pop_front();
        }
        ctx.recent_events.push_back(log_snap);
    }

    /// Increments a metric counter.
    pub fn increment_metric(&self, metric: &str) {
        let mut ctx = self.context.lock().unwrap();
        match metric {
            "messages_sent" => ctx.metrics.messages_sent += 1,
            "messages_delivered" => ctx.metrics.messages_delivered += 1,
            "timers_fired" => ctx.metrics.timers_fired += 1,
            "faults_injected" => ctx.metrics.faults_injected += 1,
            _ => {}, // Unknown metric, ignore
        }
    }

    /// Builds a snapshot of the world, enriching it with telemetry context.
    pub fn build_snapshot(&self, world: &World, time: SimTime) -> Snapshot {
        let ctx = self.context.lock().unwrap();
        let nodes = world
            .nodes
            .iter()
            .enumerate()
            .map(|(i, n)| {
                let kv = ctx.node_kvs.get(i).cloned().unwrap_or_default();
                snapshot::NodeSnap {
                    id: n.id,
                    status: n.status,
                    timers: n.timers_len(),
                    byzantine: n.byzantine(),
                    custom: kv,
                }
            })
            .collect();

        let links = world
            .net
            .links
            .values()
            .map(|l| snapshot::LinkSnap {
                id: l.id,
                src: l.src,
                dst: l.dst,
                is_partitioned: l.faults.partitioned,
            })
            .collect();

        Snapshot {
            time,
            nodes,
            links,
            recent_events: ctx.recent_events.iter().cloned().collect(),
            metrics: ctx.metrics.clone(),
        }
    }
}
