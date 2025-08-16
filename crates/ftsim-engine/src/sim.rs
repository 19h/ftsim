//! # ftsim-engine::sim
//!
//! This file contains the `Simulation` struct, which is the main entry point
//! and orchestrator for the entire simulation. It holds the master clock,
//! the event queue, the world state, and the deterministic RNG. The `step`
//! method forms the core of the discrete-event simulation loop.

use crate::{
    control::{ControlMsg, SimulationState},
    events::{Event, EventDiscriminant, FaultEventInternal, Queued},
    ids::IdGen,
    prelude::*,
    rng::{Recorder, RngDiscipline},
    store::{StoreFaultModel, StoreView},
    world::World,
};
use ftsim_proto::api::{LogIndex, LogRecord};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use std::collections::BinaryHeap;

/// The main simulation controller.
pub struct Simulation {
    /// The current simulation time. Monotonically increasing.
    clock: SimTime,
    /// The priority queue of all scheduled future events.
    queue: BinaryHeap<Queued<Event>>,
    /// The state of all nodes, the network, and storage.
    world: World,
    /// The central source of all randomness.
    rng: ChaCha20Rng,
    /// A helper for generating unique, monotonic IDs.
    pub id_gen: IdGen,
    /// The bus for sending logs, metrics, and snapshots.
    telemetry: TelemetryBus,
    /// Records all deterministic decisions for auditing and replay.
    recorder: Recorder,
    /// The current simulation state (running, paused, etc.).
    state: SimulationState,
    /// Receiver for control messages from the TUI.
    control_rx: Option<crossbeam_channel::Receiver<ControlMsg>>,
}

impl Simulation {
    /// Creates a new simulation instance.
    pub fn new(seed: u64, world: World, telemetry: TelemetryBus) -> Self {
        let rng = ChaCha20Rng::seed_from_u64(seed);
        let recorder = Recorder::new(seed);

        Self {
            clock: SIM_EPOCH,
            queue: BinaryHeap::new(),
            world,
            rng,
            id_gen: IdGen::new(),
            telemetry,
            recorder,
            state: SimulationState::Running,
            control_rx: None,
        }
    }

    /// Sets the control channel receiver for receiving messages from the TUI.
    pub fn set_control_channel(&mut self, rx: crossbeam_channel::Receiver<ControlMsg>) {
        self.control_rx = Some(rx);
    }

    /// Initializes all protocol instances on all nodes.
    /// This must be called after the simulation is created but before `run`.
    pub fn init(&mut self) {
        let node_ids: Vec<NodeId> = (0..self.world.nodes.len() as u32).collect();
        for nid in node_ids {
            // Use a separate scope to ensure borrows don't overlap
            self.init_node(nid);
        }
    }

    fn init_node(&mut self, node_id: NodeId) {
        // Use raw pointers to avoid borrow conflicts
        let node_ptr = self.world.node_mut(node_id) as *mut crate::node::runtime::Node;
        let sim_ptr = self as *mut Simulation;

        unsafe {
            let mut ctx = EngineCtx {
                sim: &mut *sim_ptr,
                current_node_id: Some(node_id),
            };

            (*node_ptr).init(&mut ctx);
        }
    }

    /// Executes a single event from the queue, advances the clock, and returns the new time.
    /// Returns `None` if the event queue is empty.
    pub fn step(&mut self) -> Option<SimTime> {
        let queued_event = self.queue.pop()?;
        let event = queued_event.payload;

        assert!(queued_event.time >= self.clock, "Time went backwards!");
        self.clock = queued_event.time;

        let event_id = queued_event.id;
        self.telemetry.set_current_time(self.clock, event_id);

        let mut ctx = EngineCtx {
            sim: self,
            current_node_id: None,
        };
        match event {
            Event::Deliver { env, link_id: _ } => {
                let dst = env.dst;
                ctx.current_node_id = Some(dst);

                // Check if this is a fault-injected message (src = u32::MAX)
                let is_fault_injected = env.src == u32::MAX;
                let payload_preview = if env.payload.len() <= 50 {
                    String::from_utf8_lossy(&env.payload).to_string()
                } else {
                    format!("{}...", String::from_utf8_lossy(&env.payload[..50]))
                };

                if is_fault_injected {
                    tracing::info!(
                        dst = env.dst,
                        msg_id = env.msg_id,
                        payload_len = env.payload.len(),
                        payload_preview = %payload_preview,
                        "📨 Fault-injected message delivered to node"
                    );
                } else {
                    tracing::info!(target: "events", src = env.src, dst = env.dst, msg_id = env.msg_id, "📨 Message delivered");
                }

                ctx.sim.telemetry.log_event(
                    if is_fault_injected { "FAULT_MESSAGE_DELIVERED" } else { "MESSAGE_DELIVERED" }.to_string(),
                    if is_fault_injected {
                        format!("Fault-injected message {} delivered to node {} (payload: '{}')", env.msg_id, env.dst, payload_preview.trim())
                    } else {
                        format!("Message {} from node {} to node {}", env.msg_id, env.src, env.dst)
                    },
                    Some(dst)
                );
                ctx.sim.telemetry.increment_metric("messages_delivered");

                // Use raw pointer to avoid double borrow
                let node_ptr = ctx.sim.world.node_mut(dst) as *mut crate::node::runtime::Node;
                unsafe {
                    (*node_ptr).handle_message(&mut ctx, env);
                }
            }
            Event::TimerFired { node_id, timer_id } => {
                ctx.current_node_id = Some(node_id);
                tracing::info!(target: "events", %node_id, %timer_id, "⏰ Timer fired");
                ctx.sim.telemetry.log_event(
                    "TIMER_FIRED".to_string(),
                    format!("Timer {} fired on node {}", timer_id, node_id),
                    Some(node_id)
                );
                ctx.sim.telemetry.increment_metric("timers_fired");
                // Use raw pointer to avoid double borrow
                let node_ptr = ctx.sim.world.node_mut(node_id) as *mut crate::node::runtime::Node;
                unsafe {
                    (*node_ptr).handle_timer(&mut ctx, timer_id);
                }
            }
            Event::Fault(fault) => {
                tracing::warn!(target: "events", ?fault, "💥 Fault injected");
                let fault_desc = match &fault {
                    FaultEventInternal::Crash { node_id, duration } => {
                        format!("Node {} crashed for {}ns", node_id, duration)
                    },
                    FaultEventInternal::Restart { node_id } => {
                        format!("Node {} restarted", node_id)
                    },
                    FaultEventInternal::Partition { sets } => {
                        format!("Network partitioned into {} sets", sets.len())
                    },
                    FaultEventInternal::HealPartition => "Network partition healed".to_string(),
                    _ => format!("{:?}", fault),
                };
                ctx.sim.telemetry.log_event(
                    "FAULT_INJECTED".to_string(),
                    fault_desc,
                    None
                );
                ctx.sim.telemetry.increment_metric("faults_injected");
                // Use a helper method to avoid borrow issues
                let sim_ptr = &mut *ctx.sim as *mut Simulation;
                unsafe {
                    (*sim_ptr).handle_fault(&mut ctx, fault);
                }
            }
            Event::UiSnapshotTick => {
                let snap = self.telemetry.build_snapshot(&self.world, self.clock);
                self.telemetry.send_snapshot(snap);
                self.schedule_at(
                    self.clock + sim_from_ms(50),
                    Event::UiSnapshotTick,
                    EventDiscriminant::ui(),
                );
            }
        }

        Some(self.clock)
    }

    /// Processes any pending control messages from the TUI.
    fn process_control_messages(&mut self) {
        // Collect messages first to avoid borrow issues
        let messages: Vec<ControlMsg> = if let Some(ref rx) = self.control_rx {
            let mut msgs = Vec::new();
            while let Ok(msg) = rx.try_recv() {
                msgs.push(msg);
            }
            msgs
        } else {
            Vec::new()
        };

        // Now process them
        for msg in messages {
            self.handle_control_message(msg);
        }
    }

    /// Handles a control message from the TUI.
    fn handle_control_message(&mut self, msg: ControlMsg) {
        match msg {
            ControlMsg::Pause => {
                tracing::info!("Simulation paused by user");
                self.state = SimulationState::Paused;
            }
            ControlMsg::Resume => {
                tracing::info!("Simulation resumed by user");
                self.state = SimulationState::Running;
            }
            ControlMsg::Step => {
                tracing::info!("Single step requested");
                self.state = SimulationState::Stepping;
            }
            ControlMsg::KillNode(node_id) => {
                tracing::info!("Killing node {} by user request", node_id);
                self.schedule_at(
                    self.clock,
                    Event::Fault(FaultEventInternal::Crash {
                        node_id,
                        duration: MAX_SIM_TIME, // Permanent crash
                    }),
                    EventDiscriminant::fault(),
                );
            }
            ControlMsg::RestartNode(node_id) => {
                tracing::info!("Restarting node {} by user request", node_id);
                self.schedule_at(
                    self.clock,
                    Event::Fault(FaultEventInternal::Restart { node_id }),
                    EventDiscriminant::fault(),
                );
            }
            ControlMsg::InjectPartition { sets } => {
                tracing::info!("Injecting network partition by user request: {:?}", sets);
                self.schedule_at(
                    self.clock,
                    Event::Fault(FaultEventInternal::Partition { sets }),
                    EventDiscriminant::fault(),
                );
            }
            ControlMsg::HealPartition => {
                tracing::info!("Healing network partition by user request");
                self.schedule_at(
                    self.clock,
                    Event::Fault(FaultEventInternal::HealPartition),
                    EventDiscriminant::fault(),
                );
            }
            ControlMsg::SetSpeed(speed) => {
                tracing::info!("Speed adjustment to {}x not yet implemented", speed);
                // TODO: Implement speed control
            }
        }
    }

    /// Runs the simulation until the event queue is empty or a stop condition is met.
    pub fn run(&mut self) {
        loop {
            // Process control messages
            self.process_control_messages();

            // Check if we should pause
            if self.state == SimulationState::Paused {
                std::thread::sleep(std::time::Duration::from_millis(50));
                continue;
            }

            // Step the simulation
            if self.step().is_none() {
                self.state = SimulationState::Completed;
                break;
            }

            // If we're in stepping mode, pause after this step
            if self.state == SimulationState::Stepping {
                self.state = SimulationState::Paused;
            }
        }
        tracing::info!("Simulation finished.");
    }

    /// Runs the simulation until a specific time is reached.
    pub fn run_until(&mut self, stop_at: SimTime) {
        loop {
            // Process control messages
            self.process_control_messages();

            // Check if we should pause
            if self.state == SimulationState::Paused {
                std::thread::sleep(std::time::Duration::from_millis(50));
                continue;
            }

            // Check if we've reached the stop time
            if let Some(queued_event) = self.queue.peek() {
                if queued_event.time > stop_at {
                    break;
                }
            }

            // Step the simulation
            if self.step().is_none() {
                self.state = SimulationState::Completed;
                break;
            }

            // If we're in stepping mode, pause after this step
            if self.state == SimulationState::Stepping {
                self.state = SimulationState::Paused;
            }
        }
        tracing::info!(stop_time = stop_at, "Simulation paused at time limit.");
    }

    /// Schedules a new event to occur at a future time.
    pub fn schedule_at(
        &mut self,
        when: SimTime,
        ev: Event,
        discriminant: EventDiscriminant,
    ) -> EventId {
        let event_id = self.id_gen.next_event_id();
        let queued_event = Queued::new(
            event_id,
            when,
            self.id_gen.next_insertion_seq(),
            discriminant,
            ev,
        );
        self.queue.push(queued_event);
        event_id
    }

    /// Returns the current simulation time.
    pub fn now(&self) -> SimTime {
        self.clock
    }

    /// Returns a reference to the telemetry bus.
    pub fn telemetry(&self) -> &TelemetryBus {
        &self.telemetry
    }

    /// Returns a reference to the world state.
    pub fn world(&self) -> &World {
        &self.world
    }

    /// Handles an internal fault event, modifying the world state.
    fn handle_fault(&mut self, ctx: &mut EngineCtx, fault: FaultEventInternal) {
        match fault {
            FaultEventInternal::Crash { node_id, duration } => {
                ctx.current_node_id = Some(node_id);
                self.world
                    .node_mut(node_id)
                    .apply_fault(ctx, fault.clone());
                // Schedule the restart if duration is not infinite
                if duration < MAX_SIM_TIME {
                    let restart_time = self.clock + duration;
                    self.schedule_at(
                        restart_time,
                        Event::Fault(FaultEventInternal::Restart { node_id }),
                        EventDiscriminant::fault(),
                    );
                }
            }
            FaultEventInternal::Restart { node_id } => {
                ctx.current_node_id = Some(node_id);
                self.world.node_mut(node_id).apply_fault(ctx, fault);
            }
            FaultEventInternal::Partition { sets } => {
                self.world.net.set_partition(sets);
            }
            FaultEventInternal::HealPartition => {
                self.world.net.heal_partition();
            }
            FaultEventInternal::ClockSkew { node_id, .. } => {
                ctx.current_node_id = Some(node_id);
                self.world.node_mut(node_id).apply_fault(ctx, fault);
            }
            FaultEventInternal::StoreFault { node_id, kind, rate } => {
                // Set the node context
                ctx.current_node_id = Some(node_id);
                // Update the store fault model
                let node = self.world.node_mut(node_id);
                match kind {
                    StoreFaultKind::FsyncFail => {
                        node.store_faults().fsync_fail_rate = rate;
                    }
                    StoreFaultKind::FsyncDelay => {
                        node.store_faults().fsync_delay_rate = rate;
                    }
                    StoreFaultKind::WriteError => {
                        node.store_faults().write_error_rate = rate;
                    }
                    StoreFaultKind::ReadError => {
                        node.store_faults().read_error_rate = rate;
                    }
                    StoreFaultKind::TornWrite => {
                        node.store_faults().torn_write_rate = rate;
                    }
                    StoreFaultKind::StaleRead => {
                        node.store_faults().stale_read_rate = rate;
                    }
                }
                // Propagate the fault to the protocol
                self.world.node_mut(node_id).apply_fault(ctx, fault);
            }
            FaultEventInternal::ByzantineFlip { node_id, enabled } => {
                ctx.current_node_id = Some(node_id);
                // Propagate the fault to the protocol and update node state
                self.world.node_mut(node_id).apply_fault(ctx, fault);
                tracing::info!(node_id, enabled, "Byzantine mode toggled");
            }
            FaultEventInternal::LinkModelUpdate { link_id, change } => {
                use crate::events::LinkModelChange;

                if let Some(link) = self.world.net.links.get_mut(&link_id) {
                    match change {
                        LinkModelChange::SetDelay(spec) => {
                            link.faults.base_delay = spec;
                            tracing::info!(link_id, ?spec, "Updated link delay");
                        }
                        LinkModelChange::SetDrop(p) => {
                            link.faults.drop = Bernoulli(p);
                            tracing::info!(link_id, p, "Updated link drop probability");
                        }
                        LinkModelChange::SetDuplicate(p) => {
                            link.faults.duplicate = Bernoulli(p);
                            tracing::info!(link_id, p, "Updated link duplicate probability");
                        }
                        LinkModelChange::SetCorrupt(p) => {
                            link.faults.corrupt = Bernoulli(p);
                            tracing::info!(link_id, p, "Updated link corruption probability");
                        }
                    }
                } else {
                    tracing::warn!(link_id, "Link not found for fault update");
                }
            }
            FaultEventInternal::BroadcastBytes { payload_hex, proto_tag } => {
                tracing::info!("🔀 Processing BroadcastBytes fault injection");
                tracing::info!(hex_payload = %payload_hex, "📦 Raw hex payload to broadcast");

                // Decode the hex payload
                match decode_hex(&payload_hex) {
                    Ok(payload_bytes) => {
                        // Convert bytes to string for logging if it's printable
                        let payload_str = String::from_utf8_lossy(&payload_bytes);
                        tracing::info!(
                            payload_len = payload_bytes.len(),
                            payload_text = %payload_str,
                            "✅ Successfully decoded hex to bytes"
                        );

                        // Send to all nodes in the simulation
                        let node_count = self.world.nodes.len();
                        tracing::info!(target_nodes = node_count, "📡 Broadcasting to all nodes in simulation");

                        for node_id in 0..node_count as u32 {
                            // Create an envelope to deliver the raw bytes
                            // Use the provided proto_tag, defaulting to 0 if none specified
                            let msg_id = self.id_gen.next_msg_id();
                            let env = Envelope {
                                src: u32::MAX, // Use max u32 to indicate system/fault injection
                                dst: node_id,
                                proto_tag: proto_tag.unwrap_or(ProtoTag(0)),
                                payload: payload_bytes.clone(),
                                msg_id,
                                create_time: self.clock,
                                trace_id: 0,
                            };

                            // Schedule immediate delivery
                            self.schedule_at(
                                self.clock,
                                Event::Deliver { env, link_id: 0 },
                                EventDiscriminant::delivery(u32::MAX),
                            );

                            tracing::info!(dst = node_id, msg_id, payload_len = payload_bytes.len(), "📨 Scheduled message delivery to node");
                        }

                        tracing::info!(
                            "🎯 BroadcastBytes fault injection completed: {} bytes sent to {} nodes",
                            payload_bytes.len(),
                            node_count
                        );

                        self.telemetry.log_event(
                            "BROADCAST_BYTES_SUCCESS".to_string(),
                            format!("Successfully broadcasted {} bytes ('{}') to {} nodes", payload_bytes.len(), payload_str.trim(), node_count),
                            None
                        );
                    }
                    Err(err) => {
                        tracing::error!(error = %err, payload_hex = %payload_hex, "❌ Failed to decode hex payload for BroadcastBytes");
                        self.telemetry.log_event(
                            "BROADCAST_BYTES_ERROR".to_string(),
                            format!("Failed to decode hex payload: {}", err),
                            None
                        );
                    }
                }
            }
            // Other custom faults are handled here.
            FaultEventInternal::Custom { name, args } => {
                tracing::warn!(name, ?args, "Custom fault handling not implemented for this type");
            }
        }
    }
}

/// Decodes a hex string into bytes.
fn decode_hex(hex_str: &str) -> Result<bytes::Bytes, String> {
    let hex_str = hex_str.trim();

    // Check if the string has an even number of hex characters
    if hex_str.len() % 2 != 0 {
        return Err(format!("Invalid hex string length: {}", hex_str.len()));
    }

    let mut bytes = Vec::with_capacity(hex_str.len() / 2);
    for i in (0..hex_str.len()).step_by(2) {
        let hex_pair = &hex_str[i..i+2];
        match u8::from_str_radix(hex_pair, 16) {
            Ok(byte) => bytes.push(byte),
            Err(_) => return Err(format!("Invalid hex characters: {}", hex_pair)),
        }
    }

    Ok(bytes::Bytes::from(bytes))
}

/// An internal context object passed to component handlers (like `Node`).
/// It provides mutable access to the simulation state.
pub struct EngineCtx<'a> {
    pub sim: &'a mut Simulation,
    pub current_node_id: Option<NodeId>,
}

impl<'a> EngineCtx<'a> {
    /// Provides a disciplined way to access the master RNG.
    pub fn rng(&mut self, site_label: &'static str) -> RngDiscipline {
        RngDiscipline::new(&mut self.sim.rng, &mut self.sim.recorder, site_label)
    }
}

/// Implementation of the `ProtoCtx` trait that the engine provides to protocols.
/// This is the bridge between the protocol's world and the engine's world.
impl<'a> ProtoCtx for EngineCtx<'a> {
    fn send_raw(&mut self, dst: NodeId, proto_tag: ProtoTag, bytes: bytes::Bytes) {
        let src = self
            .current_node_id
            .expect("Cannot send without a source node context");
        let msg_id = self.sim.id_gen.next_msg_id();
        let env = Envelope {
            src,
            dst,
            proto_tag,
            payload: bytes,
            msg_id,
            create_time: self.sim.clock,
            trace_id: 0, // TODO: Implement tracing correlation
        };
        tracing::debug!(src, dst, msg_id, "📤 Sending message");
        self.sim.telemetry.log_event(
            "MESSAGE_SENT".to_string(),
            format!("Message {} sent from node {} to node {}", msg_id, src, dst),
            Some(src)
        );
        self.sim.telemetry.increment_metric("messages_sent");
        // Use raw pointer to avoid double borrow
        let net_ptr = &mut self.sim.world.net as *mut crate::net::Net;
        unsafe {
            (*net_ptr).send(self, env);
        }
    }

    fn broadcast_raw(
        &mut self,
        proto_tag: ProtoTag,
        bytes: bytes::Bytes,
        filter: Option<&dyn Fn(NodeId) -> bool>,
    ) {
        let src = self
            .current_node_id
            .expect("Cannot broadcast without a source node context");
        let peers = self.sim.world.node(src).peers().to_vec(); // Avoid borrow issues
        for dst in peers {
            if dst != src && filter.map_or(true, |f| f(dst)) {
                self.send_raw(dst, proto_tag, bytes.clone());
            }
        }
    }

    fn set_timer(&mut self, after: SimTime) -> TimerId {
        let node_id = self
            .current_node_id
            .expect("Cannot set a timer without a node context");
        // Use raw pointer to avoid double borrow
        let node_ptr = self.sim.world.node_mut(node_id) as *mut crate::node::runtime::Node;
        unsafe {
            (*node_ptr).set_timer(self, after)
        }
    }

    fn cancel_timer(&mut self, timer_id: TimerId) -> bool {
        let node_id = self
            .current_node_id
            .expect("Cannot cancel a timer without a node context");
        self.sim.world.node_mut(node_id).cancel_timer(timer_id)
    }

    fn now(&self) -> SimTime {
        let node_id = self
            .current_node_id
            .expect("Cannot get time without a node context");
        let base = self.sim.clock;
        let skew = self.sim.world.node(node_id).clock_skew_ns;

        if skew >= 0 {
            base.saturating_add(skew as u128)
        } else {
            base.saturating_sub((-skew) as u128)
        }
    }

    fn node_id(&self) -> NodeId {
        self.current_node_id.expect("No node context")
    }

    fn store(&mut self) -> Box<dyn ftsim_proto::api::StoreView + '_> {
        let node_id = self.node_id();
        // Use raw pointers to avoid double mutable borrow
        let node_ptr = self.sim.world.node_mut(node_id) as *mut crate::node::runtime::Node;
        unsafe {
            // Get raw pointers to the store components from the node
            let view_ptr = (*node_ptr).store_view() as *mut dyn StoreView;
            let faults_ptr = (*node_ptr).store_faults() as *mut StoreFaultModel;
            Box::new(EngineStoreWrapper {
                view: &mut *view_ptr,
                faults: &mut *faults_ptr,
                ctx: self,
                node_id,
            })
        }
    }

    fn rng_u64(&mut self) -> u64 {
        use rand::Rng;
        let site = Box::leak(format!("proto.node[{}]", self.node_id()).into_boxed_str());
        self.rng(site).gen()
    }

    fn log_kv(&mut self, key: &'static str, val: &str) {
        // Convert the string to a JSON value for consistency with telemetry
        let json_val = serde_json::Value::String(val.to_string());
        self.sim.telemetry.log_node_kv(self.node_id(), key.to_string(), json_val);
    }
}

/// A simple wrapper that bridges the engine's StoreView to the protocol's StoreView.
struct EngineStoreWrapper<'a, 'b> {
    view: &'a mut dyn StoreView,
    faults: &'a mut StoreFaultModel,
    ctx: &'a mut EngineCtx<'b>,
    node_id: NodeId,
}

impl ftsim_proto::api::StoreView for EngineStoreWrapper<'_, '_> {
    fn append_log(&mut self, rec: LogRecord) -> Result<LogIndex, StoreError> {
        use rand::Rng;
        let node_id = self.node_id;

        if self.faults.write_error_rate > 0.0 {
            let site = Box::leak(format!("store.append.write_error.node[{}]", node_id).into_boxed_str());
            if self.ctx.rng(site).gen_bool(self.faults.write_error_rate) {
                tracing::warn!(%node_id, "Injecting write error in append_log");
                return Err(StoreError::FaultInjected);
            }
        }

        if self.faults.torn_write_rate > 0.0 {
            let site = Box::leak(format!("store.append.torn_write.node[{}]", node_id).into_boxed_str());
            if self.ctx.rng(site).gen_bool(self.faults.torn_write_rate) {
                tracing::warn!(%node_id, "Injecting torn write in append_log");
                return Err(StoreError::FaultInjected);
            }
        }

        self.view.append_log(rec)
    }

    fn read_log(&mut self, idx: LogIndex) -> Result<Option<LogRecord>, StoreError> {
        use rand::Rng;
        let node_id = self.node_id;

        if self.faults.read_error_rate > 0.0 {
            let site = Box::leak(format!("store.read.read_error.node[{}]", node_id).into_boxed_str());
            if self.ctx.rng(site).gen_bool(self.faults.read_error_rate) {
                tracing::warn!(%node_id, "Injecting read error in read_log");
                return Err(StoreError::FaultInjected);
            }
        }

        if self.faults.stale_read_rate > 0.0 {
            let site = Box::leak(format!("store.read.stale_read.node[{}]", node_id).into_boxed_str());
            if self.ctx.rng(site).gen_bool(self.faults.stale_read_rate) {
                tracing::warn!(%node_id, "Injecting stale read in read_log");
                return Ok(None);
            }
        }

        self.view.read_log(idx)
    }

    fn kv_put(&mut self, k: bytes::Bytes, v: bytes::Bytes) -> Result<(), StoreError> {
        self.view.kv_put(k, v)
    }

    fn kv_get(&mut self, k: &[u8]) -> Result<Option<bytes::Bytes>, StoreError> {
        self.view.kv_get(k)
    }

    fn fsync(&mut self) -> Result<(), StoreError> {
        // Inject faults like FaultyStoreView does
        use rand::Rng;
        let node_id = self.node_id;
        let site = Box::leak(format!("store.fsync.node[{}]", node_id).into_boxed_str());
        if self.ctx.rng(site).gen_bool(self.faults.fsync_fail_rate) {
            tracing::warn!(%node_id, "Injecting fsync failure");
            return Err(StoreError::FaultInjected);
        }
        self.view.fsync()
    }
}
