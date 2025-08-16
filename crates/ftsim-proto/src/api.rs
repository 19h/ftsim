//! # ftsim-proto::api
//!
//! Defines the core traits for protocol implementations. It separates the
//! user-facing typed API (`Protocol<M>`) from the engine-facing dynamic
//! trait object API (`ProtocolDyn`).

use ftsim_types::{
    envelope::ProtoTag,
    errors::CodecError,
    id::{NodeId, TimerId},
    scenario::StoreFaultKind,
};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

// --- Engine-Facing Trait ---

/// The dynamic, object-safe trait that the simulation engine uses to interact
/// with any protocol. It operates on raw byte slices.
pub trait ProtocolDyn: Send {
    /// Returns the static name of the protocol.
    fn name(&self) -> &'static str;

    /// Returns the unique tag for this protocol's messages.
    fn proto_tag(&self) -> ProtoTag;

    /// Called once when the node is initialized.
    fn init(&mut self, ctx: &mut dyn ProtoCtx);

    /// Called when a message is received from another node.
    fn on_message(
        &mut self,
        ctx: &mut dyn ProtoCtx,
        src: NodeId,
        bytes: &[u8],
    ) -> Result<(), CodecError>;

    /// Called when a previously set timer fires.
    fn on_timer(&mut self, ctx: &mut dyn ProtoCtx, timer: TimerId);

    /// Called when a fault is injected into the node by the simulator.
    fn on_fault(&mut self, ctx: &mut dyn ProtoCtx, fault: FaultEvent);
}

// --- Protocol-Author-Facing Trait ---

/// The ergonomic, typed trait that protocol authors should implement.
/// It is generic over the protocol's message type `M`.
pub trait Protocol<M>: Send
where
    M: DeserializeOwned + Serialize + Debug + Send + 'static,
{
    /// Returns the static name of the protocol.
    fn name(&self) -> &'static str;

    /// Returns the unique tag for this protocol's messages.
    fn proto_tag(&self) -> ProtoTag;

    /// Called once when the node is initialized.
    fn init(&mut self, ctx: &mut super::ctx_ext::Ctx<M>);

    /// Called when a message is received and successfully deserialized.
    fn on_message(&mut self, ctx: &mut super::ctx_ext::Ctx<M>, src: NodeId, msg: M);

    /// Called when a previously set timer fires.
    fn on_timer(&mut self, ctx: &mut super::ctx_ext::Ctx<M>, timer: TimerId);

    /// Called when a fault is injected into the node by the simulator.
    fn on_fault(&mut self, ctx: &mut super::ctx_ext::Ctx<M>, fault: FaultEvent);
}

// --- Adapter to bridge Protocol<M> to ProtocolDyn ---

struct ProtocolAdapter<P, M>
where
    P: Protocol<M>,
    M: DeserializeOwned + Serialize + Debug + Send + 'static,
{
    inner: P,
    _phantom: std::marker::PhantomData<M>,
}

impl<P, M> ProtocolDyn for ProtocolAdapter<P, M>
where
    P: Protocol<M> + Send,
    M: DeserializeOwned + Serialize + Debug + Send + 'static,
{
    fn name(&self) -> &'static str {
        self.inner.name()
    }

    fn proto_tag(&self) -> ProtoTag {
        self.inner.proto_tag()
    }

    fn init(&mut self, ctx: &mut dyn ProtoCtx) {
        let tag = self.inner.proto_tag();
        let mut wrapped_ctx = super::ctx_ext::Ctx::<M>::new(ctx, tag);
        self.inner.init(&mut wrapped_ctx);
    }

    fn on_message(
        &mut self,
        ctx: &mut dyn ProtoCtx,
        src: NodeId,
        bytes: &[u8],
    ) -> Result<(), CodecError> {
        let msg: M = postcard::from_bytes(bytes)
            .map_err(|e| CodecError(format!("Deserialization failed: {}", e)))?;
        let tag = self.inner.proto_tag();
        let mut wrapped_ctx = super::ctx_ext::Ctx::<M>::new(ctx, tag);
        self.inner.on_message(&mut wrapped_ctx, src, msg);
        Ok(())
    }

    fn on_timer(&mut self, ctx: &mut dyn ProtoCtx, timer: TimerId) {
        let tag = self.inner.proto_tag();
        let mut wrapped_ctx = super::ctx_ext::Ctx::<M>::new(ctx, tag);
        self.inner.on_timer(&mut wrapped_ctx, timer);
    }

    fn on_fault(&mut self, ctx: &mut dyn ProtoCtx, fault: FaultEvent) {
        let tag = self.inner.proto_tag();
        let mut wrapped_ctx = super::ctx_ext::Ctx::<M>::new(ctx, tag);
        self.inner.on_fault(&mut wrapped_ctx, fault);
    }
}

/// A helper function to erase the concrete message type of a `Protocol<M>`
/// implementation, returning a `Box<dyn ProtocolDyn>` that the engine can manage.
pub fn boxed_dyn<P, M>(p: P) -> Box<dyn ProtocolDyn>
where
    P: Protocol<M> + 'static,
    M: DeserializeOwned + Serialize + Debug + Send + 'static,
{
    Box::new(ProtocolAdapter {
        inner: p,
        _phantom: std::marker::PhantomData,
    })
}

// --- Engine-Provided Context Trait ---

/// This trait defines the interface that the simulation engine provides to protocols.
/// It is the "other half" of the API, representing the capabilities of the simulator
/// that a protocol can invoke (side effects).
pub trait ProtoCtx {
    fn send_raw(&mut self, dst: NodeId, proto_tag: ProtoTag, bytes: bytes::Bytes);
    fn broadcast_raw(
        &mut self,
        proto_tag: ProtoTag,
        bytes: bytes::Bytes,
        filter: Option<&dyn Fn(NodeId) -> bool>,
    );
    fn set_timer(&mut self, after: ftsim_types::time::SimTime) -> TimerId;
    fn cancel_timer(&mut self, timer: TimerId) -> bool;
    fn now(&self) -> ftsim_types::time::SimTime;
    fn node_id(&self) -> NodeId;
    fn store(&mut self) -> Box<dyn StoreView + '_>;
    fn rng_u64(&mut self) -> u64;
    fn log_kv(&mut self, key: &'static str, val: &str);
}

/// A view into the node's persistent storage.
pub trait StoreView {
    fn append_log(&mut self, rec: LogRecord) -> Result<LogIndex, ftsim_types::errors::StoreError>;
    fn read_log(&mut self, idx: LogIndex) -> Result<Option<LogRecord>, ftsim_types::errors::StoreError>;
    fn kv_put(
        &mut self,
        k: bytes::Bytes,
        v: bytes::Bytes,
    ) -> Result<(), ftsim_types::errors::StoreError>;
    fn kv_get(&mut self, k: &[u8]) -> Result<Option<bytes::Bytes>, ftsim_types::errors::StoreError>;
    fn fsync(&mut self) -> Result<(), ftsim_types::errors::StoreError>;
}

#[derive(Clone, Debug)]
pub struct LogRecord {
    pub term: u64,
    pub data: bytes::Bytes,
}
pub type LogIndex = u64;

// --- Fault Events ---

/// An event representing a fault injected by the simulator.
#[derive(Clone, Debug)]
pub enum FaultEvent {
    NodeCrashed,
    NodeRecovered,
    Partitioned { peers: Vec<NodeId> },
    PartitionHealed,
    ClockSkewed { skew_ns: i128 },
    StoreFaulted { kind: StoreFaultKind },
    ByzantineEnabled(bool),
}
