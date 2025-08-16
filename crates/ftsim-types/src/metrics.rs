//! # ftsim-types::metrics
//!
//! Defines constants for metric names and labels.
//! Centralizing these constants prevents typos and ensures consistency
//! between the engine (where metrics are emitted) and consumers (like the TUI
//! or external dashboards).

// --- Metric Names ---
pub const MET_NET_MSG_SENT: &str = "ftsim_net_msg_sent_total";
pub const MET_NET_MSG_DELIVERED: &str = "ftsim_net_msg_delivered_total";
pub const MET_NET_MSG_DROPPED: &str = "ftsim_net_msg_dropped_total";
pub const MET_TIMER_FIRED: &str = "ftsim_timer_fired_total";
pub const MET_NODE_CRASHED: &str = "ftsim_node_crashed_total";
pub const MET_NODE_RESTARTED: &str = "ftsim_node_restarted_total";
pub const MET_STORE_WRITE_ERR: &str = "ftsim_store_write_errors_total";
pub const MET_LATENCY_HISTO: &str = "ftsim_net_latency_ns";
pub const MET_EVENT_EXEC_HISTO: &str = "ftsim_event_exec_ns";
pub const MET_NODES_UP_GAUGE: &str = "ftsim_nodes_up";
pub const MET_LINKS_PARTITIONED_GAUGE: &str = "ftsim_links_partitioned";

// --- Label Keys ---
pub const LBL_NODE: &str = "node";
pub const LBL_SRC: &str = "src";
pub const LBL_DST: &str = "dst";
pub const LBL_LINK: &str = "link";
pub const LBL_EVENT: &str = "event";
pub const LBL_PROTO: &str = "proto";
pub const LBL_REASON: &str = "reason";
pub const LBL_KIND: &str = "kind";
